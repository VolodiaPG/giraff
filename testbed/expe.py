import asyncio
import json
import math
import os
import random
import traceback
from dataclasses import dataclass
from datetime import datetime
from typing import List

import aiohttp  # type: ignore
import dill  # type: ignore
import numpy as np  # type: ignore

from function import IMAGE_REGISTRY, FunctionPipeline, load_function_descriptions

port_int = int
MiB_int = int
millicpu_int = int
ms_int = int
secs_int = int


@dataclass
class Function:
    target_node: str
    mem: MiB_int
    cpu: millicpu_int
    latency: ms_int
    duration: ms_int
    docker_fn_name: str
    function_name: str
    first_node_ip: str | None
    request_interval: ms_int
    arrival: secs_int
    req_content: str
    cold_start_overhead: ms_int
    stop_overhead: ms_int
    input_max_size: str

@dataclass
class FunctionProvisioned:
    faas_ip: str
    faas_port: int
    function_id: str
    node_id: str


OFFLINE_MODE = os.getenv("OFFLINE_MODE", "") == "true"

RANDOM_SEED = os.getenv("RANDOM_SEED")
if RANDOM_SEED is not None and RANDOM_SEED != "":
    random.seed(int(RANDOM_SEED))


def generate_rand(min: int, max: int) -> int:
    # Generate a random number of nbytes
    if RANDOM_SEED is None:
        random_number = os.urandom(4)

        # Convert the bytes to an integer
        random_integer = int.from_bytes(random_number, byteorder="big")

        return (random_integer % (max - min + 1)) + min

    return random.randint(min, max)


def open_loop_poisson_process(nb, period):
    nb+=1
    # Generate X inter-arrival times based on exponential distribution with mean 1
    inter_arrival_times = np.random.exponential(1, nb)
    # Scale the inter-arrival times so their sum equals T
    scale_factor = period / np.sum(inter_arrival_times)
    scaled_inter_arrival_times = inter_arrival_times * scale_factor
    # Calculate the actual arrival times as the cumulative sum of the scaled inter-arrival times
    arrival_times = np.cumsum(scaled_inter_arrival_times)
    arrival_times=arrival_times[:-1]

    return arrival_times

if __name__ == "__main__":
    for k, v in os.environ.items():
        if k in [
            "TARGET_NODES",
            "TARGET_NODE_NAMES",
            "IOT_IP",
            "MARKET_IP",
            "MARKET_LOCAL_PORT",
            "IOT_LOCAL_PORT",
            "EXPE_SAVE_FILE",
            "EXPE_LOAD_FILE",
            "DOCKER_REGISTRY"
        ]:
            print(f"{k}={v}")

TARGET_NODES = os.getenv("TARGET_NODES", "").split()
TARGET_NODE_NAMES = os.getenv(
    "TARGET_NODE_NAMES", ""
).split()  # Should be in the same order than TARGET_NODES

IOT_IP = os.getenv("IOT_IP")
MARKET_IP = os.getenv("MARKET_IP")
MARKET_LOCAL_PORT = port_int(os.environ["MARKET_LOCAL_PORT"])
IOT_LOCAL_PORT = port_int(os.environ["IOT_LOCAL_PORT"])
NODES_IP = os.getenv("NODES_IP")

NO_LATENCY = ms_int(os.environ["NO_LATENCY"])
HIGH_LATENCY = ms_int(os.environ["HIGH_LATENCY"])
LOW_LATENCY = ms_int(os.environ["LOW_LATENCY"])

FUNCTION_COLD_START_OVERHEAD = ms_int(os.environ["FUNCTION_COLD_START_OVERHEAD"])
FUNCTION_STOP_OVERHEAD = ms_int(os.environ["FUNCTION_STOP_OVERHEAD"])
EXPERIMENT_DURATION = secs_int(os.environ["EXPERIMENT_DURATION"])

# Debug function for running local
OVERRIDE_FUNCTION_IP = os.getenv("OVERRIDE_FUNCTION_IP")
print(f"OVERRIDE_FUNCTION_IP={OVERRIDE_FUNCTION_IP}")
OVERRIDE_FIRST_NODE_IP = os.getenv("OVERRIDE_FIRST_NODE_IP")
print(f"OVERRIDE_FIRST_NODE_IP={OVERRIDE_FIRST_NODE_IP}")

# IMAGE_REGISTRY is the src and the dest used during run time is DOCKER_REGISTRY
DOCKER_REGISTRY=os.getenv("DOCKER_REGISTRY")

class AsyncSession:
    def __init__(self):
        self.timeout = aiohttp.ClientTimeout()

    async def __aenter__(self):
        self.session = aiohttp.ClientSession(timeout=self.timeout)
        return self.session

    async def __aexit__(self, exc_type, exc_value, traceback):
        await self.session.close()


async def put_request_fog_node(function: Function):
    url = f"http://{MARKET_IP}:{MARKET_LOCAL_PORT}/api/function"
    headers = {"Content-Type": "application/json"}
    data = {
        "sla": {
            "memory": f"{function.mem} MB",
            "cpu": f"{function.cpu} millicpu",
            "latencyMax": f"{function.latency} ms",
            "maxReplica": 1,
            "duration": f"{function.duration} ms",
            "functionImage": f"{DOCKER_REGISTRY}/{function.docker_fn_name}",
            "functionLiveName": f"{function.function_name}",
            "dataFlow": [  # TODO: update because outdated
                {
                    "from": {"dataSource": f"{function.target_node}"},
                    "to": "thisFunction",
                }
            ],
            "inputMaxSize": function.input_max_size,
        },
        "targetNode": f"{function.target_node}",
    }
    print(data)
    async with AsyncSession() as session:
        async with session.put(url, headers=headers, json=data) as response:
            http_code = response.status
            response = await response.content.read()
            return response, http_code


async def provision_one_function(function_id: str):
    url = f"http://{MARKET_IP}:{MARKET_LOCAL_PORT}/api/function/{function_id}"
    async with AsyncSession() as session:
        async with session.post(url) as response:
            http_code = response.status
            response = await response.content.read()

    return response, http_code


async def post_provision_chain_functions(urls: List[FunctionProvisioned]):
    return await asyncio.gather(
        *[provision_one_function(url.function_id) for url in urls]
    )


async def post_request_chain_functions(urls: List[FunctionProvisioned]):
    headers = {"Content-Type": "application/json"}
    last = len(urls) - 1
    if last == 0:
        return None

    ret = []
    for ii in range(0, last):
        urls[ii].faas_ip
        if OVERRIDE_FUNCTION_IP is not None:
            pass
        data = {
            "nextFunctionUrl": f"http://{urls[ii+1].faas_ip}:{urls[ii+1].faas_port}/function/fogfn-{urls[ii+1].function_id}",
        }
        # Sync request
        url = f"http://{urls[ii].faas_ip}:{urls[ii].faas_port}/function/fogfn-{urls[ii].function_id}/reconfigure"

        print(
            "from:",
            urls[ii].function_id,
            "- to:",
            urls[ii + 1].function_id,
            "- url:",
            url,
            "- data:",
            data,
        )

        await asyncio.sleep(1)

        try:
            async with AsyncSession() as session:
                async with session.post(url, headers=headers, json=data) as response:
                    http_code = response.status
                    response = await response.content.read()

                    ret.append((response, http_code))
        except Exception as e:
            print(f"Something went wrong contacting openfaas on {url}: {e}")
            return []

    return ret

async def put_request_iot_emulation(
    provisioned: FunctionProvisioned,
    function: Function,
):
    faas_ip = function.first_node_ip
    if OVERRIDE_FIRST_NODE_IP is not None:
        faas_ip = OVERRIDE_FIRST_NODE_IP

    url = f"http://{IOT_IP}:{IOT_LOCAL_PORT}/api/cron"
    headers = {"Content-Type": "application/json"}
    data = {
        "iotUrl": f"http://{IOT_IP}:{IOT_LOCAL_PORT}/api/print",
        "nodeUrl": f"http://{provisioned.faas_ip}:{provisioned.faas_port}/function/fogfn-{provisioned.function_id}",
        "functionId": provisioned.function_id,
        "tags": function.function_name,
        "initialWaitMs": function.cold_start_overhead,
        "durationMs": function.duration - function.stop_overhead,
        "intervalMs": function.request_interval,
        "firstNodeIp": faas_ip,
        "content": function.req_content,
    }

    async with AsyncSession() as session:
        async with session.put(url, headers=headers, json=data) as response:
            http_code = response.status
            response = await response.content.read()
            return response, http_code


async def register_new_functions(functions: List[Function]) -> bool:
    await asyncio.sleep(functions[0].arrival)

    responses = []
    started_at = None
    for ii in range(0, len(functions)):
        function = functions[ii]
        response, code = await asyncio.ensure_future(put_request_fog_node(function))
        if code != 200:
            #print("Fog request somehow failed", code, response)
            return False
        if started_at is None:
            started_at = datetime.now()
        response = json.loads(response)
        faas_ip = response["chosen"]["ip"]
        node_id = response["chosen"]["bid"]["nodeId"]
        faas_port = response["chosen"]["port"]
        function_id = response["sla"]["id"]
        response = FunctionProvisioned(faas_ip, faas_port, function_id, node_id)
        responses.append(response)
        if ii + 1 < len(functions):
            functions[ii + 1].target_node = response.node_id
            functions[ii + 1].first_node_ip = response.faas_ip

        print(f"Reserving... {ii+1}/{len(functions)}")
    print(f"Reserved {','.join([ff.function_name for ff in functions])}")

    duration = functions[0].duration
    if (
        started_at is None
        or (datetime.now() - started_at).microseconds / 1000 * 2 > duration
    ):
        print("Got the reservation, but no time to proceed to use it, stopping there")
        return False

    responses_chain = await post_provision_chain_functions(responses)
    if responses_chain is None:
        print("Failed to provision: none returned")
        return False
    for response, http_code in responses_chain:
        if http_code != 200:
            print("Provisioning failed", http_code)
            return False
    print(f"Provisioned {','.join([ff.function_name for ff in functions])}")

    if OFFLINE_MODE:
        return True


    responses_chain = await post_request_chain_functions(responses)
    if responses_chain is None:
        print("Failed to chain functions: none returned")
        return False
    for response, http_code in responses_chain:
        if http_code != 200:
            print("Request failed", http_code, response)
            return False
    print(f"Chained {','.join([ff.function_name for ff in functions])}")

    response_iot, code_iot = await put_request_iot_emulation(
        responses[0],
        function=functions[0],
    )

    if code_iot == 200:
        print(f"Registered cron for {functions[0].function_name}")
        return True

    print("---\n", response_iot.decode("utf-8").replace("\\n", "\n"))

    return False


successes = 0
errors = 0


async def do_request(functions: List[Function]):
    global successes
    global errors
    success = False
    try:
        success = await register_new_functions(functions)
    except Exception:
        traceback.print_exc()
    if success:
        successes += 1
    else:
        errors += 1


PERCENTILE_NORMAL_LOW = -2
PERCENTILE_NORMAL_HIGH = 2

async def save_file(filename: str):
    functions: List[List[Function]] = []

    function_descriptions = load_function_descriptions()
    nb_functions = []
    for fn_desc in function_descriptions:
        nb_functions.append(int(os.getenv(fn_desc.nbVarName, "0")))
    for target_node_name in TARGET_NODE_NAMES:
        for ii, fn_desc in enumerate(function_descriptions):
            nb_function = nb_functions[ii]
            # latencies = [
            #     max(1, math.ceil(x)) for x in np.random.normal(70, 30.0, nb_function)
            # ]
            request_intervals = [
                math.ceil(abs(100 * x))
                for x in np.random.gamma(2.35, 15, nb_function)
                # math.ceil(abs(1000 * x)) for x in np.random.gamma(0.75, 47, nb_function)
            ]
            durations = [
                #    math.ceil(100000 * x)
                60*4
                for x in np.random.lognormal(-0.38, 2.36, nb_function)
            ]
            arrivals = [
                math.ceil(x)
                for x in open_loop_poisson_process(nb_function,EXPERIMENT_DURATION)
            ]

            for index in range(0, nb_function):
                # latency = latencies[index]
                arrival = arrivals[index]
                duration = durations[index]
                duration = (
                    duration + FUNCTION_COLD_START_OVERHEAD + FUNCTION_STOP_OVERHEAD
                )
                request_interval = request_intervals[index]

                fn_name = list(fn_desc.pipeline.keys())[0]

                fn_chain = list()
                while True:
                    fn: FunctionPipeline = fn_desc.pipeline[fn_name]
                    latency = int(os.getenv(fn_desc.pipeline[fn_name].latency, "-1"))
                    latency = math.ceil(abs(np.random.normal(latency, latency / 4)))
                    function_name = (
                        f"{fn_name}"
                        f"-i{index}"
                        f"-c{fn.cpu}"
                        f"-m{fn.mem}"
                        f"-l{latency}"
                        f"-a{arrival}"
                        f"-r{request_interval}"
                        f"-d{duration}"
                    )
                    fn_chain.append(
                        Function(
                            target_node=target_node_name,
                            mem=fn.mem,
                            cpu=fn.cpu,
                            latency=latency,
                            duration=duration,
                            request_interval=request_interval,
                            function_name=function_name,
                            docker_fn_name=fn.image,
                            first_node_ip=None,
                            arrival=arrival,
                            req_content=fn_desc.content,
                            cold_start_overhead=FUNCTION_COLD_START_OVERHEAD,
                            stop_overhead=FUNCTION_STOP_OVERHEAD,
                            input_max_size=fn.input_max_size,
                        )
                    )

                    if fn.nextFunction is None:
                        break
                    fn_name = fn.nextFunction
                functions.append(fn_chain)

    with open(filename, "wb") as outp:  # Overwrites any existing file.
        dill.dump(functions, outp, dill.HIGHEST_PROTOCOL)


def load_functions(filename) -> List[List[Function]]:
    functions = []
    with open(filename, "rb") as inp:
        functions = dill.load(inp)
    return functions

async def load_file(filename: str):
    nodes = {}
    for ii in range(len(TARGET_NODE_NAMES)):
        nodes[TARGET_NODE_NAMES[ii].replace("'", "")] = TARGET_NODES[ii].replace(
            "'", ""
        )
    functions = load_functions(filename)
    tasks = []

    response = None
    async with AsyncSession() as session:
        async with session.get(
            f"http://{MARKET_IP}:{MARKET_LOCAL_PORT}/api/fog"
        ) as response:
            response.status
            response = await response.json()
    fognet = {fog_node_data["id"]: fog_node_data["ip"] for fog_node_data in response}

    for function_sublist in functions:
        # Use the id of the node instead of its name
        for function in function_sublist:
            function.target_node = nodes[function.target_node.replace("'", "")]
            function.first_node_ip = (
                NODES_IP if NODES_IP else fognet[function.target_node]
            )
        tasks.append(
            asyncio.create_task(
                do_request(functions=function_sublist)
            )
        )

    await asyncio.gather(*tasks)


async def main():
    env_save_file = os.getenv("EXPE_SAVE_FILE")
    env_load_file = os.getenv("EXPE_LOAD_FILE")
    if env_save_file is not None:
        await save_file(env_save_file)
    elif env_load_file is not None:
        assert len(TARGET_NODES) == len(TARGET_NODE_NAMES)
        assert(IMAGE_REGISTRY is not None)
        global DOCKER_REGISTRY
        if DOCKER_REGISTRY is None:
            DOCKER_REGISTRY=IMAGE_REGISTRY
        assert(DOCKER_REGISTRY is not None)

        print(f"Using market ({MARKET_IP}) and iot_emulation({IOT_IP})")
        await load_file(env_load_file)
        print(f"--> Did {successes}, failed to provision {errors} functions.")
    else:
        print("Not EXPE_SAVE_FILE nor EXPE_LOAD_FILE were passed, aborting")
        return 1


if __name__ == "__main__":
    asyncio.run(main())
