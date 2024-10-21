import asyncio
import json
import os
from dataclasses import dataclass
from typing import List, Optional

import marshmallow_dataclass  # type: ignore

FUNCTION_DESCRIPTIONS = os.getenv("FUNCTION_DESCRIPTIONS", "").split()  # Should be in the same order than TARGET_NODES


port_int = int
MiB_int = int
millicpu_int = int
ms_int = int
secs_int = int


@dataclass
class FunctionPipeline:
    image: str
    nextFunction: Optional[str] = None
    mem: MiB_int = 256
    cpu: millicpu_int = 100
    latency: str = "NO_LATENCY"
    input_max_size: str = "1500 B"
    expectedRequestIntervalMs: float


@dataclass
class FunctionPipelineDescription:
    name: str
    first: str
    content: str
    nbVarName: str
    pipeline: dict[str, FunctionPipeline]


IMAGE_REGISTRY = os.environ["IMAGE_REGISTRY"]


def load_function_descriptions() -> List[FunctionPipelineDescription]:
    ret = []
    schema = marshmallow_dataclass.class_schema(FunctionPipelineDescription)()
    assert len(FUNCTION_DESCRIPTIONS) != 0
    for desc_file in FUNCTION_DESCRIPTIONS:
        with open(desc_file) as ff:
            ret.append(schema.load(json.load(ff)))
    return ret

async def _build_push_function(function: str, docker_registry: str):
    print(f"Building and copying to docker://{docker_registry}/{function}")
    max_retries = 3
    retry_delay = 5  # seconds
    
    for attempt in range(max_retries):
        try:
            nix_function = function.split(":")[1]
            process = await asyncio.create_subprocess_exec(
                "nix",
                "build",
                "--print-out-paths",
                "--experimental-features",
                "nix-command flakes",
                "--no-link",
                "--quiet",
                f".#{nix_function}",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await asyncio.wait_for(process.communicate(), timeout=300)  # 5 minutes timeout
            if process.returncode == 0:
                print(f"Successfully built {function}")
            else:
                print(f"Error building {function}: {stderr.decode()}")

            function_path = stdout.decode().strip()

            read, write = os.pipe()

            await asyncio.create_subprocess_exec(
                function_path,
                stdout=write,
                stderr=asyncio.subprocess.PIPE,
            )
            os.close(write)
            process = await asyncio.create_subprocess_exec(
                "skopeo",
                "copy",
                "--insecure-policy",
                "--tls-verify=false",
                "--quiet",
                "--retry-times=3",
                "docker-archive:/dev/stdin",
                f"docker://{docker_registry}/{function}",
                stdin=read,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            os.close(read)

            stdout, stderr = await asyncio.wait_for(process.communicate(), timeout=300)  # 5 minutes timeout
            if process.returncode == 0:
                print(f"Successfully copied {function}")
                break
            else:
                print(f"Error copying {function} from {function_path} to docker://{docker_registry}/{function}: {stderr.decode()}")
        except asyncio.TimeoutError:
            print(f"Timeout while copying {function}")
        except Exception as e:
            print(f"Unexpected error while copying {function}: {str(e)}")
        
        if attempt < max_retries - 1:
            print(f"Retrying in {retry_delay} seconds...")
            await asyncio.sleep(retry_delay)
    else:
        print(f"Failed to copy {function} after {max_retries} attempts")


async def _push_function(function: str, docker_registry: str):
    print(f"Copying from docker://{IMAGE_REGISTRY}/{function} to docker://{docker_registry}/{function}")
    max_retries = 3
    retry_delay = 5  # seconds
    
    for attempt in range(max_retries):
        try:
            process = await asyncio.create_subprocess_exec(
                "skopeo",
                "copy",
                "--insecure-policy",
                "--tls-verify=false",
                "--quiet",
                "--retry-times=3",
                f"docker://{IMAGE_REGISTRY}/{function}",
                f"docker://{docker_registry}/{function}",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await asyncio.wait_for(process.communicate(), timeout=300)  # 5 minutes timeout
            if process.returncode == 0:
                print(f"Successfully copied {function}")
                break
            else:
                print(f"Error copying {function}: {stderr.decode()}")
        except asyncio.TimeoutError:
            print(f"Timeout while copying {function}")
        except Exception as e:
            print(f"Unexpected error while copying {function}: {str(e)}")
        
        if attempt < max_retries - 1:
            print(f"Retrying in {retry_delay} seconds...")
            await asyncio.sleep(retry_delay)
    else:
        print(f"Failed to copy {function} after {max_retries} attempts")

async def _do_functions_to_registry(pipeline: List[FunctionPipelineDescription], nodes: List[str], callback):
    assert IMAGE_REGISTRY is not None

    unique_functions = set()
    for pipe in pipeline:
        for desc in pipe.pipeline.values():
            unique_functions.add(desc.image)

    node_targets = set(nodes)

    tasks = [callback(function, f"{target}:5555") for target in node_targets for function in unique_functions]
    
    # Use a semaphore to limit concurrent tasks
    semaphore = asyncio.Semaphore(50)

    async def bounded_push(task):
        async with semaphore:
            await task

    # Create tasks with semaphore
    bounded_tasks = [asyncio.create_task(bounded_push(task)) for task in tasks]

    # Wait for all tasks to complete
    await asyncio.gather(*bounded_tasks)

    print("All uploads of functions to the internal container registry are done")


async def push_functions_to_registry(pipeline: List[FunctionPipelineDescription], nodes: List[str]):
    await _do_functions_to_registry(pipeline, nodes, _push_function)

async def build_functions_to_registry(pipeline: List[FunctionPipelineDescription], nodes: List[str]):
    await _do_functions_to_registry(pipeline, nodes, _build_push_function)

