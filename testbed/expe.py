import asyncio
import json
import math
import os
import random
from dataclasses import dataclass
from typing import Any, List

import aiohttp  # type: ignore
import dill  # type: ignore
import numpy as np  # type: ignore
from alive_progress import alive_bar  # type: ignore

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
    cold_start_overhead: ms_int
    stop_overhead: ms_int
    duration: ms_int
    docker_fn_name: str
    function_name: str
    first_node_ip: str | None
    request_interval: ms_int
    # load_type: str
    arrival: secs_int


@dataclass
class FunctionProvisioned:
    faas_ip: str
    faas_port: int
    function_id: str
    node_id: str


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


def open_loop_poisson_process(rate, time):
    """
    Simulates an open-loop Poisson process.

    Parameters:
    rate: The average rate of events (lambda).
    time: The total time to simulate.

    Returns:
    A list of event times.
    """
    num_events = np.random.poisson(rate * time)
    event_times = np.random.uniform(0, time, num_events)

    # # Sort the event times
    # event_times.sort()

    return event_times


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
        ]:
            print(f"{k}={v}")

TARGET_NODES = os.getenv("TARGET_NODES", "").split()
TARGET_NODE_NAMES = os.getenv(
    "TARGET_NODE_NAMES", ""
).split()  # Shoud be in the same order than TARGET_NODES
if len(TARGET_NODES) != 0:
    assert len(TARGET_NODES) == len(TARGET_NODE_NAMES)

IOT_IP = os.getenv("IOT_IP")
MARKET_IP = os.getenv("MARKET_IP")
MARKET_LOCAL_PORT = port_int(os.environ["MARKET_LOCAL_PORT"])
IOT_LOCAL_PORT = port_int(os.environ["IOT_LOCAL_PORT"])
NODES_IP = os.getenv("NODES_IP")
IMAGE_REGISTRY = os.environ["IMAGE_REGISTRY"]
FUNCTION_NAME = os.environ["FUNCTION_NAME"]

FUNCTION_MEMORY = MiB_int(os.environ["FUNCTION_MEMORY"])  # MiB
FUNCTION_CPU = millicpu_int(os.environ["FUNCTION_CPU"])  # Millicpu

NO_LATENCY = ms_int(os.environ["NO_LATENCY"])
HIGH_LATENCY = ms_int(os.environ["HIGH_LATENCY"])
LOW_LATENCY = ms_int(os.environ["LOW_LATENCY"])

NB_FUNCTIONS = int(os.environ["NB_FUNCTIONS"])

FUNCTION_COLD_START_OVERHEAD = ms_int(os.environ["FUNCTION_COLD_START_OVERHEAD"])
FUNCTION_STOP_OVERHEAD = ms_int(os.environ["FUNCTION_STOP_OVERHEAD"])
EXPERIMENT_DURATION = secs_int(os.environ["EXPERIMENT_DURATION"])

# Debug function for running local
OVERRIDE_FUNCTION_IP = os.getenv("OVERRIDE_FUNCTION_IP")

print(f"OVERRIDE_FUNCTION_IP={OVERRIDE_FUNCTION_IP}")


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
    duration = function.cold_start_overhead + function.duration + function.stop_overhead
    data = {
        "sla": {
            "memory": f"{function.mem} MB",
            "cpu": f"{function.cpu} millicpu",
            "latencyMax": f"{function.latency} ms",
            "maxReplica": 1,
            "duration": f"{duration} ms",
            "functionImage": f"{IMAGE_REGISTRY}/{function.docker_fn_name}:latest",
            "functionLiveName": f"{function.function_name}",
            "dataFlow": [  # TODO: update because outdated
                {
                    "from": {"dataSource": f"{function.target_node}"},
                    "to": "thisFunction",
                }
            ],
        },
        "targetNode": f"{function.target_node}",
    }
    async with AsyncSession() as session:
        async with session.put(url, headers=headers, json=data) as response:
            http_code = response.status
            response = await response.content.read()
            return response, http_code


async def post_request_chain_functions(urls: List[FunctionProvisioned]):
    headers = {"Content-Type": "application/json"}
    last = len(urls) - 1
    if last == 0:
        return None

    # await asyncio.sleep(FUNCTION_COLD_START_OVERHEAD / 2)

    # print(urls)

    ret = []
    for ii in range(0, last):
        faas_ip = urls[ii].faas_ip
        if OVERRIDE_FUNCTION_IP is not None:
            faas_ip = OVERRIDE_FUNCTION_IP
        data = {
            "nextFunctionUrl": f"http://{faas_ip}:{urls[ii+1].faas_port}/function/fogfn-{urls[ii+1].function_id}",
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
        async with AsyncSession() as session:
            async with session.post(url, headers=headers, json=data) as response:
                http_code = response.status
                response = await response.content.read()

                ret.append((response, http_code))

    return ret


async def put_request_iot_emulation(
    provisioned: FunctionProvisioned,
    function: Function,
):
    url = f"http://{IOT_IP}:{IOT_LOCAL_PORT}/api/cron"
    headers = {"Content-Type": "application/json"}
    data = {
        "iotUrl": f"http://{IOT_IP}:{IOT_LOCAL_PORT}/api/print",
        "nodeUrl": f"http://{provisioned.faas_ip}:{provisioned.faas_port}/function/fogfn-{provisioned.function_id}",
        "functionId": provisioned.function_id,
        "tag": function.function_name,
        "initialWaitMs": function.cold_start_overhead,
        "durationMs": function.duration,
        "intervalMs": function.request_interval,
        "firstNodeIp": function.first_node_ip,
    }

    async with AsyncSession() as session:
        async with session.put(url, headers=headers, json=data) as response:
            http_code = response.status
            response = await response.content.read()
            return response, http_code


async def register_new_functions(functions: List[Function]) -> bool:
    await asyncio.sleep(functions[0].arrival)

    responses = []
    for ii in range(0, len(functions)):
        function = functions[ii]
        response, code = await asyncio.ensure_future(put_request_fog_node(function))
        if code != 200:
            return False

        response = json.loads(response)
        faas_ip = response["chosen"]["ip"]
        node_id = response["chosen"]["bid"]["nodeId"]
        faas_port = response["chosen"]["port"]
        function_id = response["chosen"]["bid"]["id"]
        response = FunctionProvisioned(faas_ip, faas_port, function_id, node_id)
        responses.append(response)
        if ii + 1 < len(functions):
            functions[ii + 1].target_node = response.node_id
            functions[ii + 1].first_node_ip = response.faas_ip

        print(f"Provisioning... {ii+1}/{len(functions)}")

    print(f"Provisioned {','.join([ff.function_name for ff in functions])}")

    tasks = []
    tasks.append(asyncio.create_task(post_request_chain_functions(responses)))
    tasks.append(
        asyncio.create_task(
            put_request_iot_emulation(
                responses[0],
                function=functions[0],
            )
        )
    )

    response_list = await asyncio.gather(*tasks)
    responses_chain = response_list[-2]
    response_iot, code_iot = response_list[-1]

    if responses_chain is not None:
        for response, http_code in responses_chain:
            if http_code != 200:
                return False

    print(f"Chained {','.join([ff.function_name for ff in functions])}")
    if code_iot == 200:
        print(f"Registered cron for {functions[0].function_name}")
        return True

    print("---\n", response_iot.decode("utf-8").replace("\\n", "\n"))

    return False


successes = 0
errors = 0


async def do_request_progress(bar: Any, functions: List[Function]):
    global successes
    global errors
    success = await register_new_functions(functions)
    if success:
        successes += 1
    else:
        errors += 1
    bar.text = f"--> Done {successes}, failed to provision {errors} functions..."
    bar()


PERCENTILE_NORMAL_LOW = -2
PERCENTILE_NORMAL_HIGH = 2


async def save_file(filename: str):
    functions: List[List[Function]] = []
    functions_raw: List[Function] = []
    docker_fn_name = FUNCTION_NAME
    for target_node_name in TARGET_NODE_NAMES:
        latencies = [
            max(1, math.ceil(x)) for x in np.random.normal(70, 30.0, NB_FUNCTIONS)
        ]
        request_intervals = [
            math.ceil(abs(1000 * x))
            for x in np.random.lognormal(-0.38, 2.36, NB_FUNCTIONS)
        ]
        durations = [
            math.ceil(1000 * x) for x in np.random.lognormal(-0.38, 2.36, NB_FUNCTIONS)
        ]
        arrivals = [
            math.ceil(x)
            for x in open_loop_poisson_process(NB_FUNCTIONS, EXPERIMENT_DURATION)
        ]

        for index in range(0, NB_FUNCTIONS):
            latency = latencies[index]
            arrival = arrivals[index]
            duration = durations[index]
            cpu = FUNCTION_CPU
            memory = FUNCTION_MEMORY

            request_interval = request_intervals[index]

            function_name = (
                f"{docker_fn_name}"
                f"-i{index}"
                f"-c{cpu}"
                f"-m{memory}"
                f"-l{latency}"
                f"-a{arrival}"
                f"-r{request_interval}"
                f"-d{duration}"
                # f"-l{load_type}"
                f"-n{NB_FUNCTIONS}"
                f"-n{NB_FUNCTIONS * len(TARGET_NODE_NAMES)}"
            )

            functions_raw.append(
                Function(
                    target_node=target_node_name,
                    mem=memory,
                    cpu=cpu,
                    latency=latency,
                    cold_start_overhead=FUNCTION_COLD_START_OVERHEAD,
                    stop_overhead=FUNCTION_STOP_OVERHEAD,
                    duration=duration,
                    request_interval=request_interval,
                    # load_type=load_type,
                    function_name=function_name,
                    docker_fn_name=docker_fn_name,
                    first_node_ip=None,
                    arrival=arrival,
                )
            )

        while len(functions_raw) != 0:
            functions_subset: List[Function] = []
            function: Function = functions_raw.pop()
            functions_subset.append(function)
            # if len(functions_raw) != 0 and random.randint(0, 3) == 0:
            # TODO: redo that
            if len(functions_raw) != 0:
                function = functions_raw.pop()
                functions_subset.append(function)

            min_duration = function.duration
            for function in functions_subset:
                min_duration = min(min_duration, function.duration)

            # TODO: remove that
            min_duration = 30000

            for function in functions_subset:
                function.duration = min_duration

            functions.append(functions_subset)

    print(functions)

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
    bar_len = len(functions)

    response = None
    async with AsyncSession() as session:
        async with session.get(
            f"http://{MARKET_IP}:{MARKET_LOCAL_PORT}/api/fog"
        ) as response:
            response.status
            response = await response.json()
    fognet = {fog_node_data["id"]: fog_node_data["ip"] for fog_node_data in response}

    with alive_bar(bar_len, title="Functions", ctrl_c=False, dual_line=True) as bar:
        for function_sublist in functions:
            # Use the id of the node instead of its name
            for function in function_sublist:
                function.target_node = nodes[function.target_node.replace("'", "")]
                function.first_node_ip = (
                    NODES_IP if NODES_IP else fognet[function.target_node]
                )
            tasks.append(
                asyncio.create_task(
                    do_request_progress(bar=bar, functions=function_sublist)
                )
            )

        await asyncio.gather(*tasks)


async def main():
    if os.getenv("EXPE_SAVE_FILE") is not None:
        await save_file(os.getenv("EXPE_SAVE_FILE"))
    elif os.getenv("EXPE_LOAD_FILE") is not None:
        print(f"Using market ({MARKET_IP}) and iot_emulation({IOT_IP})")
        await load_file(os.getenv("EXPE_LOAD_FILE"))
        print(f"--> Did {successes}, failed to provision {errors} functions.")

    else:
        print("Not EXPE_SAVE_FILE nor EXPE_LOAD_FILE were passed, aborting")
        return 1


if __name__ == "__main__":
    asyncio.run(main())
