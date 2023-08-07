import asyncio
from contextlib import contextmanager
from dataclasses import dataclass
import json
import os
import random
import pickle
import sys
from typing import Any

import aiohttp
from alive_progress import alive_bar


@dataclass
class Function:
    target_node: str
    mem: int
    cpu: int
    latency: int
    sleep_before_start: int
    docker_fn_name: str
    function_name: str
    request_interval: int


RANDOM_SEED = os.getenv("RANDOM_SEED")
if RANDOM_SEED is not None and RANDOM_SEED != "":
    random.seed(int(RANDOM_SEED))


def generate_rand(min, max):
    # Generate a random number of nbytes
    if RANDOM_SEED is None:
        random_number = os.urandom(4)

        # Convert the bytes to an integer
        random_integer = int.from_bytes(random_number, byteorder="big")

        return (random_integer % (max - min + 1)) + min

    return random.randint(min, max)


if __name__ == "__main__":
    for k, v in os.environ.items():
        if k in [
            "TARGET_NODES",
            "TARGET_NODE_NAMES",
            "IOT_IP",
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
MARKET_LOCAL_PORT = int(os.getenv("MARKET_LOCAL_PORT", 8088))
IOT_LOCAL_PORT = int(os.getenv("IOT_LOCAL_PORT", 3003))

FUNCTION_MEMORY_REQ_INTERVAL_LOW_LOW_LATENCY = int(
    os.getenv("FUNCTION_MEMORY_REQ_INTERVAL_LOW", 100)
)  # MiB
FUNCTION_CPU_REQ_INTERVAL_LOW_LOW_LATENCY = int(
    os.getenv("FUNCTION_CPU_REQ_INTERVAL_LOW", 500)
)  # Millicpu
FUNCTION_MEMORY_REQ_INTERVAL_HIGH_LOW_LATENCY = int(
    os.getenv("FUNCTION_MEMORY_REQ_INTERVAL_HIGH", 50)
)  # MiB
FUNCTION_CPU_REQ_INTERVAL_HIGH_LOW_LATENCY = int(
    os.getenv("FUNCTION_CPU_REQ_INTERVAL_HIGH", 50)
)  # Millicpu
FUNCTION_MEMORY_REQ_INTERVAL_LOW_REST_LATENCY = int(
    os.getenv("FUNCTION_MEMORY_REQ_INTERVAL_LOW", 100)
)  # MiB
FUNCTION_CPU_REQ_INTERVAL_LOW_REST_LATENCY = int(
    os.getenv("FUNCTION_CPU_REQ_INTERVAL_LOW", 500)
)  # Millicpu
FUNCTION_MEMORY_REQ_INTERVAL_HIGH_REST_LATENCY = int(
    os.getenv("FUNCTION_MEMORY_REQ_INTERVAL_HIGH", 50)
)  # MiB
FUNCTION_CPU_REQ_INTERVAL_HIGH_REST_LATENCY = int(
    os.getenv("FUNCTION_CPU_REQ_INTERVAL_HIGH", 50)
)  # Millicpu

MAX_LATENCY_LOW_LATENCY = int(os.getenv("MAX_LATENCY_LOW_LATENCY", 10))
MIN_LATENCY_LOW_LATENCY = int(os.getenv("MIN_LATENCY_LOW_LATENCY", 5))
NB_FUNCTIONS_LOW_LATENCY = int(os.getenv("NB_FUNCTIONS_LOW_LATENCY", 50))

MAX_LATENCY_REST_LATENCY = int(os.getenv("MAX_LATENCY_REST_LATENCY", 75))
MIN_LATENCY_REST_LATENCY = int(os.getenv("MIN_LATENCY_REST_LATENCY", 45))
NB_FUNCTIONS_REST = int(os.getenv("NB_FUNCTIONS_REST", 50))

NB_FUNCTIONS_LOW_REQ_INTERVAL_LOW_LATENCY = int(
    os.getenv("NB_FUNCTIONS_LOW_REQ_INTERVAL_LOW_LATENCY", 50)
)
LOW_REQ_INTERVAL = int(
    os.getenv("LOW_REQ_INTERVAL", 10)
)  # ms interval between two requests

NB_FUNCTIONS_HIGH_REQ_INTERVAL_LOW_LATENCY = int(
    os.getenv("NB_FUNCTIONS_HIGH_REQ_INTERVAL_LOW_LATENCY", 50)
)
HIGH_REQ_INTERVAL = int(
    os.getenv("HIGH_REQ_INTERVAL", 60)
)  # ms interval between two requests

FUNCTION_RESERVATION_DURATION = int(os.getenv("FUNCTION_RESERVATION_DURATION", 60))  # s
FUNCTION_LOAD_DURATION = int(os.getenv("FUNCTION_LOAD_DURATION", 55))  # s
FUNCTION_LOAD_STARTS_AFTER = int(os.getenv("FUNCTION_LOAD_STARTS_AFTER", 60))  # s
FUNCTION_RESERVATION_FINISHES_AFTER = int(
    os.getenv("FUNCTION_RESERVATION_FINISHES_AFTER", 15)
)

NB_FUNCTIONS_LOW_REQ_INTERVAL_REST_LATENCY = (
    NB_FUNCTIONS_LOW_LATENCY - NB_FUNCTIONS_LOW_REQ_INTERVAL_LOW_LATENCY
)
assert (
    NB_FUNCTIONS_LOW_REQ_INTERVAL_REST_LATENCY >= 0
), "NB_FUNCTIONS_LOW_REQ_INTERVAL_REST_LATENCY is negative"

NB_FUNCTIONS_HIGH_REQ_INTERVAL_REST_LATENCY = (
    NB_FUNCTIONS_REST - NB_FUNCTIONS_HIGH_REQ_INTERVAL_LOW_LATENCY
)
assert (
    NB_FUNCTIONS_HIGH_REQ_INTERVAL_REST_LATENCY >= 0
), "NB_FUNCTIONS_HIGH_REQ_INTERVAL_REST_LATENCY is negative"

function_latencies = []

for ii in range(NB_FUNCTIONS_HIGH_REQ_INTERVAL_LOW_LATENCY):
    function_latencies.append(
        (
            generate_rand(MIN_LATENCY_LOW_LATENCY, MAX_LATENCY_LOW_LATENCY),
            HIGH_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_HIGH_LOW_LATENCY,
            FUNCTION_CPU_REQ_INTERVAL_HIGH_LOW_LATENCY,
            "low",
            "high",
        )
    )
for ii in range(NB_FUNCTIONS_LOW_REQ_INTERVAL_LOW_LATENCY):
    function_latencies.append(
        (
            generate_rand(MIN_LATENCY_LOW_LATENCY, MAX_LATENCY_LOW_LATENCY),
            LOW_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_LOW_LOW_LATENCY,
            FUNCTION_CPU_REQ_INTERVAL_LOW_LOW_LATENCY,
            "low",
            "low",
        )
    )
for ii in range(NB_FUNCTIONS_HIGH_REQ_INTERVAL_REST_LATENCY):
    function_latencies.append(
        (
            generate_rand(MIN_LATENCY_REST_LATENCY, MAX_LATENCY_REST_LATENCY),
            HIGH_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_HIGH_REST_LATENCY,
            FUNCTION_CPU_REQ_INTERVAL_HIGH_REST_LATENCY,
            "high",
            "high",
        )
    )
for ii in range(NB_FUNCTIONS_LOW_REQ_INTERVAL_REST_LATENCY):
    function_latencies.append(
        (
            generate_rand(MIN_LATENCY_REST_LATENCY, MAX_LATENCY_REST_LATENCY),
            LOW_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_LOW_REST_LATENCY,
            FUNCTION_CPU_REQ_INTERVAL_LOW_REST_LATENCY,
            "high",
            "low",
        )
    )


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
            "duration": f"{FUNCTION_RESERVATION_DURATION} seconds",
            "functionImage": f"ghcr.io/volodiapg/{function.docker_fn_name}:latest",
            "functionLiveName": f"{function.function_name}",
            "dataFlow": [
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


async def put_request_iot_emulation(
    faas_ip: str,
    faas_port: int,
    function_id: str,
    function: Function,
    duration: int,
):
    url = f"http://{IOT_IP}:{IOT_LOCAL_PORT}/api/cron"
    headers = {"Content-Type": "application/json"}
    data = {
        "iotUrl": f"http://{IOT_IP}:{IOT_LOCAL_PORT}/api/print",
        "nodeUrl": f"http://{faas_ip}:{faas_port}/function/fogfn-{function_id}",
        "functionId": function_id,
        "tag": function.function_name,
        "initialWaitMs": FUNCTION_LOAD_STARTS_AFTER * 1000,
        "durationMs": duration * 1000,
        "intervalMs": function.request_interval,
    }

    async with AsyncSession() as session:
        async with session.put(url, headers=headers, json=data) as response:
            http_code = response.status
            response = await response.content.read()
            return response, http_code


async def register_new_function(function: Function) -> bool:
    await asyncio.sleep(function.sleep_before_start)

    response, code = await asyncio.ensure_future(put_request_fog_node(function))

    if code == 200:
        # print(f"[{request_interval_type}][{latency_type}] Provisioned {function_name}")
        try:
            data = json.loads(response)
            faas_ip = data["chosen"]["ip"]
            faas_port = data["chosen"]["port"]
            function_id = data["chosen"]["bid"]["id"]

            response, code = await asyncio.ensure_future(
                put_request_iot_emulation(
                    faas_ip=faas_ip,
                    faas_port=faas_port,
                    function_id=function_id,
                    function=function,
                    duration=FUNCTION_LOAD_DURATION,
                )
            )

            if code == 200:
                # print(
                #     f"[{request_interval_type}][{latency_type}] Registered cron for {function_name}"
                # )
                return True
        except json.JSONDecodeError:
            pass

    print("---\n", response.decode("utf-8").replace("\\n", "\n"))

    return False


successes = 0
errors = 0


async def do_request_progress(bar: Any, function: Function):
    global successes
    global errors
    success = await register_new_function(function)
    if success:
        successes += 1
    else:
        errors += 1
    bar.text = f"--> Done {successes}, failed to provision {errors} functions..."
    bar()


async def save_file(filename: str):
    functions = []
    for target_node_name in TARGET_NODE_NAMES:
        index = 0
        for function in function_latencies:
            (
                latency,
                request_interval,
                memory,
                cpu,
                latency_type,
                request_interval_type,
            ) = function
            sleep_before_start = generate_rand(0, FUNCTION_RESERVATION_FINISHES_AFTER)

            docker_fn_name = "echo"
            function_name = (
                f"{docker_fn_name}"
                f"-{index}"
                f"-{latency}"
                f"-{cpu}"
                f"-{memory}"
                f"-{request_interval_type}"
                f"-{latency_type}"
                f"-{NB_FUNCTIONS_LOW_REQ_INTERVAL_LOW_LATENCY}"
                f"-{NB_FUNCTIONS_HIGH_REQ_INTERVAL_LOW_LATENCY}"
            )

            functions.append(
                Function(
                    target_node=target_node_name,
                    mem=memory,
                    cpu=cpu,
                    latency=latency,
                    function_name=function_name,
                    docker_fn_name=docker_fn_name,
                    request_interval=request_interval,
                    sleep_before_start=sleep_before_start,
                )
            )

            index += 1

    with open(filename, "wb") as outp:  # Overwrites any existing file.
        pickle.dump(functions, outp, pickle.HIGHEST_PROTOCOL)


def load_functions(filename):
    sys.modules["__main__"].Function = Function
    functions = []
    with open(filename, "rb") as inp:
        functions = pickle.load(inp)
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
    with alive_bar(bar_len, title="Functions", ctrl_c=False, dual_line=True) as bar:
        for function in functions:
            # Use the id of the node instead of its name
            function.target_node = nodes[function.target_node.replace("'", "")]
            tasks.append(
                asyncio.create_task(do_request_progress(bar=bar, function=function))
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
