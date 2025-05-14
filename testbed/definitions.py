import copy
import heapq
import math
import os
import random
import sys
from collections import defaultdict
from typing import Any, Callable, Dict, List, Optional, Tuple

import dill  # type: ignore
import randomname  # type: ignore

ONE_GBIT = 1_000_000_000
ONE_MBIT = 1_000_000
ONE_KBIT = 1_000

RANDOM_SEED = os.getenv("RANDOM_SEED")
if RANDOM_SEED is not None and RANDOM_SEED != "":
    random.seed(int(RANDOM_SEED))


PING_REQUEST_TIMEOUT_SEC = os.getenv("PING_REQUEST_TIMEOUT_SEC", "30")

FOG_NODE_DEPLOYMENT = """apiVersion: v1
kind: ServiceAccount
metadata:
  name: fog-node
  namespace: openfaas
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: fog-node
  namespace: openfaas
rules:
  - apiGroups: ["metrics.k8s.io", ""]
    resources: ["pods", "nodes"]
    verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: fog-node-service-creation
  namespace: openfaas-fn
rules:
  - apiGroups: [""]
    resources: ["services"]
    verbs: ["create"]
---
kind: ClusterRoleBinding
apiVersion: rbac.authorization.k8s.io/v1
metadata:
  name: fog-node
  namespace: openfaas
subjects:
- kind: ServiceAccount
  name: fog-node
  namespace: openfaas
roleRef:
  kind: ClusterRole
  name: fog-node
  apiGroup: rbac.authorization.k8s.io
---
kind: RoleBinding
apiVersion: rbac.authorization.k8s.io/v1
metadata:
  name: fog-node-service-creation
  namespace: openfaas-fn
subjects:
- kind: ServiceAccount
  name: fog-node
  namespace: openfaas
roleRef:
  kind: Role
  name: fog-node-service-creation
  apiGroup: rbac.authorization.k8s.io
---
apiVersion: v1
kind: Service
metadata:
  name: fog-node
  namespace: openfaas
  labels:
    app: fog-node
spec:
  type: NodePort
  ports:
    - name: proxied-fog-node-30003
      port: 30003
      targetPort: 30003
      protocol: TCP
      nodePort: 30003
  selector:
    app: fog-node
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fog-node
  namespace: openfaas
  labels:
    app: fog-node
spec:
  replicas: 1
  selector:
    matchLabels:
      app: fog-node
  template:
    metadata:
      labels:
        app: fog-node
    spec:
      shareProcessNamespace: true
      serviceAccountName: fog-node
      automountServiceAccountToken: true
      containers:
      - name: fog-node
        image: {fog_node_image}
        imagePullPolicy: Always
        env:
        - name: OPENFAAS_USERNAME
          valueFrom:
            secretKeyRef:
              name: basic-auth
              key: basic-auth-user
        - name: OPENFAAS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: basic-auth
              key: basic-auth-password
        - name: OPENFAAS_IP
          value: "gateway.openfaas"
        - name: OPENFAAS_PORT
          value: "31112"
        - name: CONFIG
          value: "{conf}"
        - name: LOG_CONFIG_PATH
          value: "/var/log"
        - name: LOG_CONFIG_FILENAME
          value: "{node_name}.log"
        - name: RUST_LOG
          value: "{rust_log}"
        - name: IS_CLOUD
          value: "{is_cloud}"
        - name: INFLUX_ADDRESS
          value: "{influx_ip}:9086"
        - name: INFLUX_TOKEN
          value: "xowyTh1iGcNAZsZeydESOHKvENvcyPaWg8hUe3tO4vPOw_buZVwOdUrqG3gwV314aYd9SWKHcxlykcQY_rwYVQ=="
        - name: INFLUX_ORG
          value: "faasfog"
        - name: INFLUX_BUCKET
          value: "faasfog"
        - name: INSTANCE_NAME
          value: "{node_name}"
        - name: COLLECTOR_IP
          value: "{collector_ip}"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT_FUNCTION
          value: "http://{influx_ip}:4317"
        - name: ENABLE_COLLECTOR
          value: "{enable_collector}"
        - name: FUNCTION_LIVE_TIMEOUT_MSECS
          value: "120000"
{additional_env_vars}
        ports:
        - containerPort: 30003
        volumeMounts:
        - name: log-storage-fog-node
          mountPath: /var/log
      - name: sidecar-logs
        image: ghcr.io/volodiapg/busybox:latest
        args: [/bin/sh, -c, 'tail -n+1 -F /mnt/log/{node_name}.log']
        volumeMounts:
        - name: log-storage-fog-node
          readOnly: true
          mountPath: /mnt/log
      volumes:
      - name: log-storage-fog-node
        emptyDir: {{}}
"""

MARKET_DEPLOYMENT = """apiVersion: v1
kind: Service
metadata:
  name: market
  namespace: openfaas
  labels:
    app: market
spec:
  type: NodePort
  ports:
    - name: proxied-market-30008
      port: 30008
      targetPort: 30008
      protocol: TCP
      nodePort: 30008
  selector:
    app: market
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: market
  namespace: openfaas
  labels:
    app: market
spec:
  replicas: 1
  selector:
    matchLabels:
      app: market
  template:
    metadata:
      labels:
        app: market
    spec:
      containers:
      - name: market
        image: {market_image}
        imagePullPolicy: Always
        ports:
        - containerPort: 30008
        - containerPort: 6831
        - containerPort: 6832
        env:
        - name: LOG_CONFIG_PATH
          value: "/var/log"
        - name: LOG_CONFIG_FILENAME
          value: "market.log"
        - name: RUST_LOG
          value: "{rust_log}"
        - name: SERVER_PORT
          value: "30008"
        - name: INFLUX_ADDRESS
          value: "{influx_ip}:9086"
        - name: INFLUX_TOKEN
          value: "xowyTh1iGcNAZsZeydESOHKvENvcyPaWg8hUe3tO4vPOw_buZVwOdUrqG3gwV314aYd9SWKHcxlykcQY_rwYVQ=="
        - name: INFLUX_ORG
          value: "faasfog"
        - name: INFLUX_BUCKET
          value: "faasfog"
        - name: COLLECTOR_IP
          value: "{collector_ip}"
        - name: ENABLE_COLLECTOR
          value: "{enable_collector}"
        - name: INSTANCE_NAME
          value: "marketplace"
        volumeMounts:
        - name: log-storage-market
          mountPath: /var/log
      - name: sidecar-logs
        image: ghcr.io/volodiapg/busybox:latest
        args: [/bin/sh, -c, 'tail -n+1 -F /mnt/log/market.log']
        volumeMounts:
        - name: log-storage-market
          readOnly: true
          mountPath: /mnt/log
      volumes:
      - name: log-storage-market
        emptyDir: {{}}
"""

MARKET_CONNECTED_NODE = """(
    situation: MarketConnected (
      market_ip: "{market_ip}",
      market_port: "30008",
    ),
    my_id: "{my_id}",
    my_public_ip: "{my_public_ip}",
    my_private_ip: "{my_public_ip}",
    my_public_port_http: "30003",
    reserved_cpu: "{reserved_cpu} cpus",
    reserved_memory: "{reserved_memory} MiB",
    tags: ["node_to_market", "{name}"],
    max_in_flight_functions_proposals: MaxInFlight({max_in_flight}),
    my_advertised_bandwidth: "{my_advertised_bandwidth}",
)

"""

NODE_CONNECTED_NODE = """(
    situation: NodeConnected (
      parent_id: "{parent_id}",
      parent_node_ip: "{parent_ip}",
      parent_node_port_http: "30003",
    ),
    my_id: "{my_id}",
    my_public_ip: "{my_public_ip}",
    my_private_ip: "{my_public_ip}",
    my_public_port_http: "30003",
    reserved_cpu: "{reserved_cpu} cpus",
    reserved_memory: "{reserved_memory} MiB",
    tags: ["node_to_node", "{name}"],
    max_in_flight_functions_proposals: MaxInFlight({max_in_flight}),
    my_advertised_bandwidth: "{my_advertised_bandwidth}",
)

"""

OPENTELEMETRY_CONFIG = """
exporters:
  otlp/jaeger:
    endpoint: http://{collector_ip}:4317
    tls:
      insecure: true

service:
  pipelines:
    metrics:
      receivers: [otlp]
      exporters: [influxdb,otlp/jaeger]
    traces:
      receivers: [otlp]
      exporters: [influxdb,otlp/jaeger]
    logs:
      receivers: [otlp]
      exporters: [influxdb,otlp/jaeger]
"""

# Remove a unit so that the hosts are not saturated
NB_CPU_PER_MACHINE_PER_CLUSTER = {
    "gros": {"core": (2 * 18), "mem": 1024 * 96},
    "paravance": {"core": (2 * 8 * 2), "mem": 1024 * 128},
    "parasilo": {"core": (2 * 8 * 2), "mem": 1024 * 128},
    # "dahu": {"core": 2 * 16 - 1, "mem": 1024 * (192 - 4)},
}

MAX_LOCATION = 3
MAX_INITIAL_PRICE = 4
SLOPE = 8


def pricing(
    location_raw: int,
):
    location = MAX_LOCATION - location_raw
    assert location >= 0

    base_price = MAX_INITIAL_PRICE / (location + 1)
    random_variation = random.uniform(-base_price, base_price)
    price = base_price + random_variation / 2
    return price


def additional_env_vars(level):
    def inner():
        ret = {
            "PRICING_CPU": SLOPE,  # for the function
            "PRICING_MEM": SLOPE,  # for the function
            "PRICING_CPU_INITIAL": pricing(level),
            "PRICING_MEM_INITIAL": pricing(level) / 2,
            "PRICING_GEOLOCATION": SLOPE,  # for already used mem and cpu
            "ELECTRICITY_PRICE": 1.0,
        }
        if level >= 3:
            ret.update(
                {
                    "RATIO_AA": random.uniform(0.8, 1.2),
                    "RATIO_BB": random.uniform(1.2, 1.4),
                    "ELECTRICITY_PRICE": 1.0,
                }
            )
        elif level == 2:
            ret.update(
                {
                    "RATIO_AA": random.uniform(0.4, 0.8),
                    "RATIO_BB": random.uniform(0.9, 1.1),
                    "ELECTRICITY_PRICE": 0.95,
                }
            )
        elif level <= 1:
            ret.update(
                {
                    "RATIO_AA": 0.01,
                    "RATIO_BB": 0.2,
                    "ELECTRICITY_PRICE": 0.75,
                }
            )
        return ret

    return inner


TIER_4_FLAVOR = {
    "core": 4,
    "mem": 1024 * 4,
    "reserved_core": 3,
    "reserved_mem": 1024 * 3,
    "additional_env_vars": additional_env_vars(3),
}
TIER_3_FLAVOR = {
    "core": 6,
    "mem": 1024 * 6,
    "reserved_core": 5,
    "reserved_mem": 1024 * 5,
    "additional_env_vars": additional_env_vars(2),
}
TIER_2_FLAVOR = {
    "core": 10,
    "mem": 1024 * 12,
    "reserved_core": 9,
    "reserved_mem": 1024 * 11,
    "additional_env_vars": additional_env_vars(1),
}
TIER_1_FLAVOR = {
    "is_cloud": True,
    "core": 16,
    "mem": 1024 * 46,
    "reserved_core": 15,
    "reserved_mem": 1024 * 42,
    "additional_env_vars": additional_env_vars(0),
}

uuid = 0


def gen_vm_conf(node):
    ret = defaultdict(lambda: [])
    children = node["children"] if "children" in node else []
    for child in children:
        ret[frozenset(child["flavor"].items())].append(child["name"])
        for key, value in gen_vm_conf(child).items():
            for val in value:
                ret[key].append(val)

    return ret


def generate_level(
    flavor: Dict,
    *,
    nb_nodes: Tuple[int, int],
    latencies: Tuple[int, int],
    rates: Tuple[int, int],
    losses: Tuple[int, int],
    modifiers: Optional[
        List[Callable[[Dict[str, Any], bool], None]]
    ] = None,  # Takes a Dict but otherwise mypy just errors on kwargs
    next_lvl: Optional[Callable[[int], List[Dict[str, Any]]]] = None,
) -> Callable[[int], List[Dict[str, Any]]]:
    def inner(depth: int = 1) -> List[Dict[str, Any]]:
        ret: List[Dict] = []
        global uuid
        first = True
        for _ in range(
            0, random.randint(math.ceil(nb_nodes[0]), math.ceil(nb_nodes[1]))
        ):
            uuid += 1
            rate_min = min(rates[0], rates[1])
            rate_max = max(rates[0], rates[1])
            loss_min = min(losses[0], losses[1])
            loss_max = max(losses[0], losses[1])
            city = {
                "name": str(depth) + randomname.get_name().replace("-", "") + str(uuid),
                "flavor": copy.copy(flavor),
                "latency": random.randint(latencies[0], latencies[1]),
                "rate": rate_min * random.randint(1, math.ceil(rate_max / rate_min)),
                "loss": loss_min * random.randint(1, math.ceil(loss_max / loss_min)),
                "children": next_lvl(depth=depth + 1) if next_lvl else [],  # type: ignore
            }
            if modifiers:
                for mod in modifiers:
                    mod(city, first)
            first = False
            ret.append(city)
        return ret

    return inner


def set_cloud(dd: Dict, *_):
    dd["is_cloud"] = True


def set_iot_connected(drop_one_in: int):
    def set_connected(dd: Dict, first: bool):
        if first or random.randint(1, drop_one_in) != 1:
            dd["iot_connected"] = 0

    return set_connected


def drop_children(drop_one_in: int):
    def drop(dd: Dict, first: bool):
        if first:
            return
        if random.randint(1, drop_one_in) == 1:
            dd["children"] = []

    return drop


def flavor_randomizer_mem(reductions: List[int]):
    def drop(dd: Dict, *_):
        diff_index = random.randint(0, len(reductions) - 1)
        diff = 1024 * reductions[diff_index]
        dd["flavor"]["mem"] -= diff
        dd["flavor"]["reserved_mem"] -= diff
        assert dd["flavor"]["mem"] != 0
        assert dd["flavor"]["reserved_mem"] != 0

    return drop


def flavor_randomizer_cpu(reductions: List[int]):
    def drop(dd: Dict, *_):
        diff_index = random.randint(0, len(reductions) - 1)
        diff = reductions[diff_index]
        dd["flavor"]["core"] -= diff
        dd["flavor"]["reserved_core"] -= diff
        assert dd["flavor"]["core"] != 0
        assert dd["flavor"]["reserved_core"] != 0

    return drop


SIZE_MULTIPLIER = float(os.getenv("SIZE_MULTIPLIER", "1")) * 0.5


def network_generation():
    return {
        "name": "market",
        "flavor": TIER_1_FLAVOR,
        "rate": ONE_GBIT,
        "loss": 0,
        "children": generate_level(
            TIER_1_FLAVOR,
            nb_nodes=(1, int(6 * SIZE_MULTIPLIER)),
            latencies=(1, 3),
            rates=(ONE_GBIT, ONE_GBIT),
            modifiers=[set_cloud, drop_children(drop_one_in=2)],
            next_lvl=generate_level(
                TIER_2_FLAVOR,
                nb_nodes=(2, int(4 * SIZE_MULTIPLIER)),
                latencies=(6, 32),
                rates=(500 * ONE_MBIT, ONE_GBIT),
                losses=(0, 0),
                modifiers=[
                    drop_children(drop_one_in=3),
                    flavor_randomizer_cpu([0, 2, 4]),
                    flavor_randomizer_mem([0, 2, 4]),
                ],
                next_lvl=generate_level(
                    TIER_3_FLAVOR,
                    nb_nodes=(3, int(8 * SIZE_MULTIPLIER)),
                    latencies=(7, 64),
                    rates=(100 * ONE_MBIT, ONE_GBIT),
                    losses=(0, 0),
                    modifiers=[
                        drop_children(drop_one_in=6),
                        flavor_randomizer_cpu([0, 2]),
                        flavor_randomizer_mem([0, 2, 4]),
                    ],
                    next_lvl=generate_level(
                        TIER_4_FLAVOR,
                        nb_nodes=(2, int(8 * SIZE_MULTIPLIER)),
                        latencies=(1, 4),
                        rates=(10 * ONE_MBIT, ONE_GBIT),
                        losses=(0, 0),
                        modifiers=[
                            set_iot_connected(drop_one_in=6),
                            flavor_randomizer_mem([0, 2]),
                        ],
                    ),
                ),
            ),
        )(
            1
        ),  # depth = 1, because python typesafety stuff wants you to repeat it:'(
    }


def pprint_network(node):
    ret = {}
    for key in ["name", "is_cloud", "latency", "iot_connected", "flavor"]:
        if key not in node:
            continue
        if key == "flavor":
            ret[key] = {
                "reserved_core": node[key]["reserved_core"],
                "reserved_mem": node[key]["reserved_mem"],
            }
        else:
            ret[key] = node[key]
    ret["children"] = []
    for child in node["children"]:
        ret["children"].append(pprint_network(child))
    return ret


def flatten(container):
    for i in container:
        if isinstance(i, list):
            for j in flatten(i):
                yield j
        else:
            yield i


def gen_fog_nodes_names(node):
    name = node["name"]

    children = node["children"] if "children" in node else []

    return [name, *[gen_fog_nodes_names(node) for node in children]]


def get_extremities_name(node):
    name = node["name"]

    children = node["children"] if "children" in node else []

    ret = [get_extremities_name(node) for node in children]
    if len(children) == 0:
        ret.append(name)

    return ret


def get_iot_connection(node):
    name = node["name"]

    children = node["children"] if "children" in node else []

    ret = [get_iot_connection(node) for node in children]
    if "iot_connected" in node:
        ret.append((name, node["iot_connected"]))
    return ret


def adjacency(node):
    children = node["children"] if "children" in node else []
    ret = {}
    ret[node["name"]] = [
        (child["name"], child["latency"], child["rate"]) for child in children
    ]
    for child in children:
        ret = {**ret, **adjacency(child)}

    return ret


def levels(node, level=0):
    children = node["children"] if "children" in node else []
    ret = {}
    ret[node["name"]] = level
    for child in children:
        ret = {**ret, **levels(child, level + 1)}

    return ret


def adjacency_undirected(node) -> dict[str, list[tuple[str, int, int, int]]]:
    ret = defaultdict(lambda: [])

    def fun(node):
        children = node["children"] if "children" in node else []
        for child in children:
            ret[node["name"]] += [
                (child["name"], child["latency"], child["rate"], child["loss"]),
            ]
            ret[child["name"]] += [
                (node["name"], child["latency"], child["rate"], child["loss"])
            ]
            fun(child)

    fun(node)
    return ret


def gen_net(nodes, callback):
    adjacency = adjacency_undirected(nodes)

    for name, latency in IOT_CONNECTION:
        # adjacency[name].append(("iot_emulation", latency))
        adjacency["iot_emulation"].append((name, latency, ONE_GBIT, 0))
    # Convert to matrix
    # Initialize a matrix

    ii = 0
    positions = {}
    for name in adjacency.keys():
        positions[name] = ii
        ii += 1

    def dijkstra(src: str):
        # Create a priority queue to store vertices that
        # are being preprocessed
        pq: list[tuple[int, str]] = []
        heapq.heappush(pq, (0, src))

        # Create a vector for distances and initialize all
        # distances as infinite (INF)
        dist: dict[str, int] = defaultdict(lambda: 1_000_000_000)
        rates: dict[str, int] = defaultdict(lambda: ONE_GBIT)
        losses: dict[str, int] = defaultdict(lambda: 0)
        dist[src] = 0
        rates[src] = ONE_GBIT
        losses[src] = 0

        while pq:
            # The first vertex in pair is the minimum distance
            # vertex, extract it from priority queue.
            # vertex label is stored in second of pair
            _d, u = heapq.heappop(pq)

            # 'i' is used to get all adjacent vertices of a
            # vertex
            for v, latency, rate, loss in adjacency[u]:
                # If there is shorted path to v through u.
                if dist[v] > dist[u] + latency:
                    # Updating distance of v
                    dist[v] = dist[u] + latency
                    rates[v] = min(rates[u], rate)
                    losses[v] = losses[u] + loss - math.floor((loss * losses[u]) / 100)
                    if losses[v] < 0 or losses[v] >= 100:
                        raise Exception(
                            f"Loss is {losses[v]} for {u} -> {v}, with params {rate} {latency} {loss}"
                        )
                    heapq.heappush(pq, (dist[v], v))

        return (dist, rates, losses)

    for node_name in adjacency.keys():
        latencies, rates, losses = dijkstra(node_name)  # modifies subtree_cumul
        for destination in latencies.keys():
            latency = latencies[destination]
            rate = rates[destination]
            loss = losses[destination]
            # print(f"{node_name} -> {destination} = {latency} ; {rate/ONE_GBIT}G")
            callback(node_name, destination, latency, rate, loss)


def get_number_vms(node, nb_cpu_per_host, mem_total_per_host):
    """
    Returns the number of hosts required to run the number of nodes
    """
    total_vm_required = 1  # the market is the first
    vms = gen_vm_conf(node)
    # add the market
    vms[frozenset(NETWORK["flavor"].items())].append("market")
    for key, value in vms.items():
        flavor = {x: y for (x, y) in key}
        core = flavor["core"]
        mem = flavor["mem"]

        core_used = 0
        mem_used = 0
        nb_vms = 0
        for vm_name in value:
            core_used += core
            mem_used += mem

            if core_used > nb_cpu_per_host or mem_used > mem_total_per_host:
                if nb_vms == 0:
                    raise Exception(
                        "The VM requires more resources than the node can provide"
                    )

                total_vm_required += 1
                core_used = 0
                mem_used = 0
                nb_vms = 0

            nb_vms += 1

        # Still an assignation left?
        if nb_vms > 0:
            total_vm_required += 1
    return total_vm_required


LOAD_NETWORK_FILE = "LOAD_NETWORK_FILE"
SAVE_NETWORK_FILE = "SAVE_NETWORK_FILE"

if os.getenv("DEV_NETWORK") == "true":
    NETWORK = {
        "name": "market",
        "flavor": TIER_4_FLAVOR,
        "rate": ONE_GBIT,
        "loss": 1,
        "children": [
            {
                "name": "node_2",
                "flavor": TIER_4_FLAVOR,
                "latency": 6,
                "rate": 1 * ONE_GBIT,
                "children": [
                    {
                        "name": "node_3",
                        "flavor": TIER_4_FLAVOR,
                        "latency": 10,
                        "rate": 1 * ONE_GBIT,
                        "children": [],
                        "iot_connected": 0,
                    },
                    {
                        "name": "node_34",
                        "flavor": TIER_4_FLAVOR,
                        "rate": 100 * ONE_MBIT,
                        "latency": 5,
                        "children": [],
                        "iot_connected": 0,
                    },
                ],
            },
        ],
    }
else:
    save_network_file = os.getenv(SAVE_NETWORK_FILE)
    load_network_file = os.getenv(LOAD_NETWORK_FILE)
    if save_network_file and load_network_file:
        raise Exception(
            f"{SAVE_NETWORK_FILE} and {LOAD_NETWORK_FILE} env var should not be set together"
        )
    elif save_network_file and not load_network_file:
        NETWORK = network_generation()
        dill.settings["recurse"] = True
        with open(save_network_file, "wb") as outp:  # Overwrites any existing file.
            dill.dump(NETWORK, outp, dill.HIGHEST_PROTOCOL)
    elif not save_network_file and load_network_file:
        with open(load_network_file, "rb") as inp:
            NETWORK = dill.load(inp)
    else:
        raise Exception(
            f"{SAVE_NETWORK_FILE} or {LOAD_NETWORK_FILE} env vars should be defined to save/load the network configuration"
        )

FOG_NODES = list(flatten([gen_fog_nodes_names(child) for child in NETWORK["children"]]))
fog_nodes_control = set(
    flatten([gen_fog_nodes_names(child) for child in NETWORK["children"]])
)
assert len(FOG_NODES) == len(
    fog_nodes_control
), "Some names are identical, each should be a uid"

EXTREMITIES = list(
    flatten([get_extremities_name(child) for child in NETWORK["children"]])
)
IOT_CONNECTION = list(
    flatten([get_iot_connection(child) for child in NETWORK["children"]])
)
ADJACENCY = adjacency(NETWORK)
LEVELS = levels(NETWORK)

MIN_NUMBER_VMS = os.getenv("MIN_NB_VMS")
if os.getenv(SAVE_NETWORK_FILE) and MIN_NUMBER_VMS:
    min_number_vms = int(MIN_NUMBER_VMS)
    nb_vms = len(FOG_NODES)
    if nb_vms < min_number_vms:
        print(f"Got nb_vms {nb_vms} < {min_number_vms}", file=sys.stderr)
        exit(122)

MAX_NUMBER_VMS = os.getenv("MAX_NB_VMS")
if os.getenv(SAVE_NETWORK_FILE) and MAX_NUMBER_VMS:
    max_number_nodes = int(MAX_NUMBER_VMS)
    failed = True
    cluster = os.getenv("CLUSTER") or ""
    nb_cpu_per_machine = NB_CPU_PER_MACHINE_PER_CLUSTER[cluster]["core"]
    mem_per_machine = NB_CPU_PER_MACHINE_PER_CLUSTER[cluster]["mem"]
    nb_nodes = get_number_vms(NETWORK, nb_cpu_per_machine, mem_per_machine)
    if nb_nodes > max_number_nodes:
        print(f"Got nb_nodes {nb_nodes} > {max_number_nodes}", file=sys.stderr)
        exit(123)
    print(
        f"Number of nodes on {cluster}: {nb_nodes} (this does not include vm such as iot_emulation)",
        file=sys.stderr,
    )


if __name__ == "__main__":
    # import pprint
    # pprint.pprint(pprint_network(NETWORK), sort_dicts=False)
    print("Number of vms:", len(FOG_NODES), file=sys.stderr)
