import asyncio
import json
import os
from dataclasses import dataclass
from typing import List, Optional

import marshmallow_dataclass

FUNCTION_DESCRIPTIONS = os.getenv(
    "FUNCTION_DESCRIPTIONS", ""
).split()  # Should be in the same order than TARGET_NODES


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


async def _push_function(function:str, docker_registry: str):
    print(f"Copying from docker://{IMAGE_REGISTRY}/{function} to docker://{docker_registry}/{function}")
    process = await asyncio.create_subprocess_exec("skopeo", "copy", "--insecure-policy", "--dest-tls-verify=false", "--quiet", f"docker://{IMAGE_REGISTRY}/{function}", f"docker://{docker_registry}/{function}",
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE)
    stdout, stderr = await process.communicate()
    print(stdout, stderr)


async def push_functions_to_registry(pipeline: List[FunctionPipelineDescription], nodes: List[str]):
    assert(IMAGE_REGISTRY is not None)

    unique_functions = set()
    for pipe in pipeline:
        print(pipe)
        for desc in pipe.pipeline.values():
            unique_functions.add(desc.image)

    node_targets = set(nodes)

    to_run = [ asyncio.create_task(_push_function(function, f"{target}:5555")) for target in node_targets for function in unique_functions]
    await asyncio.gather(*to_run)

    print("All upload of function to the internal container registry are done")


