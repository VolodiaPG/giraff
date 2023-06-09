import asyncio
import json
import os
import random

import aiohttp
from alive_progress import alive_bar

for k, v in os.environ.items():
    if k in ["TARGET_NODES", "IOT_IP", "MARKET_LOCAL_PORT", "IOT_LOCAL_PORT"]:
        print(f"{k}={v}")

TARGET_NODES = os.getenv("TARGET_NODES").split()
IOT_IP = os.getenv("IOT_IP")
MARKET_LOCAL_PORT = int(os.getenv("MARKET_LOCAL_PORT", 8088))
IOT_LOCAL_PORT = int(os.getenv("IOT_LOCAL_PORT", 3003))

FUNCTION_MEMORY_REQ_INTERVAL_LOW = int(
    os.getenv("FUNCTION_MEMORY_REQ_INTERVAL_LOW", 100)
)  # MiB
FUNCTION_CPU_REQ_INTERVAL_LOW = int(
    os.getenv("FUNCTION_CPU_REQ_INTERVAL_LOW", 500)
)  # Millicpu
FUNCTION_MEMORY_REQ_INTERVAL_HIGH = int(
    os.getenv("FUNCTION_MEMORY_REQ_INTERVAL_HIGH", 50)
)  # MiB
FUNCTION_CPU_REQ_INTERVAL_HIGH = int(
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
            random.randint(MIN_LATENCY_LOW_LATENCY, MAX_LATENCY_LOW_LATENCY),
            HIGH_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_HIGH,
            FUNCTION_CPU_REQ_INTERVAL_HIGH,
            "low",
            "high",
        )
    )
for ii in range(NB_FUNCTIONS_LOW_REQ_INTERVAL_LOW_LATENCY):
    function_latencies.append(
        (
            random.randint(MIN_LATENCY_LOW_LATENCY, MAX_LATENCY_LOW_LATENCY),
            LOW_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_LOW,
            FUNCTION_CPU_REQ_INTERVAL_LOW,
            "low",
            "low",
        )
    )
for ii in range(NB_FUNCTIONS_HIGH_REQ_INTERVAL_REST_LATENCY):
    function_latencies.append(
        (
            random.randint(MIN_LATENCY_REST_LATENCY, MAX_LATENCY_REST_LATENCY),
            HIGH_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_HIGH,
            FUNCTION_CPU_REQ_INTERVAL_HIGH,
            "high",
            "high",
        )
    )
for ii in range(NB_FUNCTIONS_LOW_REQ_INTERVAL_REST_LATENCY):
    function_latencies.append(
        (
            random.randint(MIN_LATENCY_REST_LATENCY, MAX_LATENCY_REST_LATENCY),
            LOW_REQ_INTERVAL,
            FUNCTION_MEMORY_REQ_INTERVAL_LOW,
            FUNCTION_CPU_REQ_INTERVAL_LOW,
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


async def put_request_fog_node(
    target_node: str,
    mem: int,
    cpu: int,
    latency: int,
    docker_fn_name: str,
    function_name: str,
):
    url = f"http://localhost:{MARKET_LOCAL_PORT}/api/function"
    headers = {"Content-Type": "application/json"}
    data = {
        "sla": {
            "memory": f"{mem} MB",
            "cpu": f"{cpu} millicpu",
            "latencyMax": f"{latency} ms",
            "maxReplica": 1,
            "duration": f"{FUNCTION_RESERVATION_DURATION} seconds",
            "functionImage": f"ghcr.io/volodiapg/{docker_fn_name}:latest",
            "functionLiveName": f"{function_name}",
            "dataFlow": [
                {"from": {"dataSource": f"{target_node}"}, "to": "thisFunction"}
            ],
        },
        "targetNode": f"{target_node}",
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
    function_name: str,
    request_interval: int,
    duration: int,
):
    url = f"http://localhost:{IOT_LOCAL_PORT}/api/cron"
    headers = {"Content-Type": "application/json"}
    data = {
        "iotUrl": f"http://{IOT_IP}:{IOT_LOCAL_PORT}/api/print",
        "nodeUrl": f"http://{faas_ip}:{faas_port}/function/fogfn-{function_id}",
        "functionId": function_id,
        "tag": function_name,
        "initialWaitMs": FUNCTION_LOAD_STARTS_AFTER * 1000,
        "durationMs": duration * 1000,
        "intervalMs": request_interval,
    }

    async with AsyncSession() as session:
        async with session.put(url, headers=headers, json=data) as response:
            http_code = response.status
            response = await response.content.read()
            return response, http_code


async def register_new_function(
    target_node: str,
    index: int,
    sleep_before_start: int,
    latency: int,
    request_interval: int,
    memory: int,
    cpu: int,
    request_interval_type: str,
    latency_type: str,
) -> bool:
    await asyncio.sleep(sleep_before_start)

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

    response, code = await asyncio.ensure_future(
        put_request_fog_node(
            target_node=target_node,
            mem=memory,
            cpu=cpu,
            latency=latency,
            docker_fn_name=docker_fn_name,
            function_name=function_name,
        )
    )

    if code == 200:
        print(f"[{request_interval_type}][{latency_type}] Provisioned {function_name}")
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
                    function_name=function_name,
                    request_interval=request_interval,
                    duration=FUNCTION_LOAD_DURATION,
                )
            )

            if code == 200:
                print(
                    f"[{request_interval_type}][{latency_type}] Registered cron for {function_name}"
                )
                return True
        except json.JSONDecodeError:
            pass

    print("---\n", response.decode("utf-8").replace("\\n", "\n"))

    return False


successes = 0
errors = 0


async def do_request_progress(
    bar: any,
    target_node: str,
    index: int,
    sleep_before_start: int,
    latency: int,
    request_interval: int,
    memory: int,
    cpu: int,
    request_interval_type: str,
    latency_type: str,
):
    global successes
    global errors
    success = await register_new_function(
        target_node=target_node,
        index=index,
        sleep_before_start=sleep_before_start,
        latency=latency,
        request_interval=request_interval,
        memory=memory,
        cpu=cpu,
        request_interval_type=request_interval_type,
        latency_type=latency_type,
    )
    if success:
        successes += 1
    else:
        errors += 1
    bar.text = f"--> Done {successes}, failed to provision {errors} functions..."
    bar()


async def main():
    tasks = []
    bar_len = len(function_latencies) * len(TARGET_NODES)
    with alive_bar(bar_len, title="Functions", ctrl_c=False, dual_line=True) as bar:
        for target_node in TARGET_NODES:
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
                sleep_before_start = random.randint(
                    0, FUNCTION_RESERVATION_FINISHES_AFTER
                )

                tasks.append(
                    asyncio.create_task(
                        do_request_progress(
                            bar=bar,
                            target_node=target_node,
                            index=index,
                            sleep_before_start=sleep_before_start,
                            latency=latency,
                            request_interval=request_interval,
                            memory=memory,
                            cpu=cpu,
                            request_interval_type=request_interval_type,
                            latency_type=latency_type,
                        )
                    )
                )
                index += 1

        await asyncio.gather(*tasks)


if __name__ == "__main__":
    asyncio.run(main())
    print(f"--> Did {successes}, failed to provision {errors} functions.")
