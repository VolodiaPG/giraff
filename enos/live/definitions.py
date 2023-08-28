import functools
import heapq
import os
import pprint
import random
from collections import defaultdict
from typing import Any, Callable, Dict, List, Tuple

import dill
import randomname

RANDOM_SEED = os.getenv("RANDOM_SEED")
if RANDOM_SEED is not None and RANDOM_SEED != "":
    random.seed(int(RANDOM_SEED))


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
          value: "warn,fog_node=trace,openfaas=trace,kube_metrics=trace,helper=trace"
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
        - name: PRICING_MEM
          value: "{pricing_mem}"
        - name: PRICING_CPU
          value: "{pricing_cpu}"
        - name: PRICING_MEM_INITIAL
          value: "{pricing_mem_initial}"
        - name: PRICING_CPU_INITIAL
          value: "{pricing_cpu_initial}"
        - name: PRICING_GEOLOCATION
          value: "{pricing_geolocation}"
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
          value: "warn,market=trace"
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
    my_public_port_http: "30003",
    reserved_cpu: "{reserved_cpu} cpus",
    reserved_memory: "{reserved_memory} MiB",
    tags: ["node_to_market", "{name}"],
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
    my_public_port_http: "30003",
    reserved_cpu: "{reserved_cpu} cpus",
    reserved_memory: "{reserved_memory} MiB",
    tags: ["node_to_node", "{name}"],
)

"""

# Remove a unit so that the hosts are not saturated
NB_CPU_PER_MACHINE_PER_CLUSTER = {
    "gros": {"core": (2 * 18) - 1, "mem": 1024 * (96 - 4)},
    "paravance": {"core": (2 * 8 * 2) - 1, "mem": 1024 * (128 - 4)},
    # "dahu": {"core": 2 * 16 - 1, "mem": 1024 * (192 - 4)},
}

MAX_LOCATION = 4
MAX_INITIAL_PRICE = 2
SLOPE = 1


def pricing(location):
    location = min(max(1, location), MAX_LOCATION)
    base_price = MAX_INITIAL_PRICE - (
        MAX_INITIAL_PRICE * (location - 1) / (MAX_LOCATION - 1)
    )

    random_variation = random.uniform(-5, 5)  # Adjust the range of variation as needed

    price = base_price + random_variation

    price = min(max(1, price), MAX_INITIAL_PRICE)

    return price


def generate_initial_pricing(location):
    return functools.partial(pricing, location)


TIER_4_FLAVOR = {
    "core": 2,
    "mem": 1024 * 4,
    "reserved_core": 1.75,
    "reserved_mem": 1024 * 3,
    "pricing_cpu": SLOPE,  # for the function
    "pricing_mem": SLOPE,  # for the function
    "pricing_cpu_initial": generate_initial_pricing(3),
    "pricing_mem_initial": generate_initial_pricing(3),
    "pricing_geolocation": SLOPE,  # for already used mem and cpu
}
TIER_3_FLAVOR = {
    "core": 4,
    "mem": 1024 * 8,
    "reserved_core": 3,
    "reserved_mem": 1024 * 7,
    "pricing_cpu": SLOPE,  # for the function
    "pricing_mem": SLOPE,  # for the function
    "pricing_cpu_initial": generate_initial_pricing(2),
    "pricing_mem_initial": generate_initial_pricing(2),
    "pricing_geolocation": SLOPE,  # for already used mem and cpu
}
TIER_2_FLAVOR = {
    "core": 6,
    "mem": 1024 * 16,
    "reserved_core": 5,
    "reserved_mem": 1024 * 14,
    "pricing_cpu": SLOPE,  # for the function
    "pricing_mem": SLOPE,  # for the function
    "pricing_cpu_initial": generate_initial_pricing(1),
    "pricing_mem_initial": generate_initial_pricing(1),
    "pricing_geolocation": SLOPE,  # for already used mem and cpu
}
TIER_1_FLAVOR = {
    "is_cloud": True,
    "core": 15,
    "mem": 1024 * 46,
    "reserved_core": 16,
    "reserved_mem": 1024 * 60,
    "pricing_cpu": SLOPE,  # for the function
    "pricing_mem": SLOPE,  # for the function
    "pricing_cpu_initial": generate_initial_pricing(0),
    "pricing_mem_initial": generate_initial_pricing(0),
    "pricing_geolocation": SLOPE,  # for already used mem and cpu
}
TIER_CLOUD_FLAVOR = {
    "is_cloud": True,
    "core": 1024,
    "mem": 1024 * 1024,
    "reserved_core": 2014,
    "reserved_mem": 1024 * 1024,
    "pricing_cpu": SLOPE,  # for the function
    "pricing_mem": SLOPE,  # for the function
    "pricing_cpu_initial": generate_initial_pricing(0),
    "pricing_mem_initial": generate_initial_pricing(0),
    "pricing_geolocation": SLOPE,  # for already used mem and cpu
}


uuid = 0


def generate_level(
    flavor: Dict,
    *,
    nb_nodes: Tuple[int, int],
    latencies: Tuple[int, int],
    modifiers: List[Callable[[Dict, bool], None]]
    | None = None,  # Takes a Dict but otherwise mypy just errors on kwargs
    next_lvl: Callable[[int], Dict] | None = None,
    depth: int = 1,
) -> List[Dict]:
    ret: List[Dict] = []
    global uuid
    first = True
    for _ in range(0, random.randint(nb_nodes[0], nb_nodes[1])):
        uuid += 1
        city = {
            "name": str(depth) + randomname.get_name().replace("-", "") + str(uuid),
            "flavor": flavor,
            "latency": random.randint(latencies[0], latencies[1]),
            "children": next_lvl(depth=depth + 1) if next_lvl else [],
        }
        if modifiers:
            for mod in modifiers:
                mod(city, first)
        first = False
        ret.append(city)
    return ret


def set_cloud(dd: Dict, *_):
    dd["is_cloud"] = True


def set_iot_connected(drop_one_in: int):
    def set_connected(dd: Dict, first: bool):
        if not first and random.randint(1, drop_one_in) == 1:
            return
        dd["iot_connected"] = 0

    return set_connected


def drop_children(drop_one_in: int):
    def drop(dd: Dict, first: bool):
        if first:
            return
        if random.randint(1, drop_one_in) == 1:
            dd["children"] = []

    return drop


SIZE_MULTIPLIER = int(os.getenv("SIZE_MULTIPLIER", "1"))


def network_generation():
    return {
        "name": "market",
        "flavor": TIER_1_FLAVOR,
        "children": generate_level(
            TIER_1_FLAVOR,
            nb_nodes=(1, 4 * SIZE_MULTIPLIER),
            latencies=(1, 3),
            modifiers=[set_cloud, drop_children(drop_one_in=2)],
            next_lvl=functools.partial(
                generate_level,
                TIER_2_FLAVOR,
                nb_nodes=(SIZE_MULTIPLIER, 2 * SIZE_MULTIPLIER),
                latencies=(1, 4),
                modifiers=[drop_children(drop_one_in=4)],
                next_lvl=functools.partial(
                    generate_level,
                    TIER_3_FLAVOR,
                    nb_nodes=(1, 2 * SIZE_MULTIPLIER),
                    latencies=(5, 60),
                    next_lvl=functools.partial(
                        generate_level,
                        TIER_4_FLAVOR,
                        nb_nodes=(1, 4 * SIZE_MULTIPLIER),
                        latencies=(1, 20),
                        modifiers=[set_iot_connected(drop_one_in=3)],
                    ),
                ),
            ),
        ),
    }


def pprint_network(node):
    ret = {}
    for key in ["name", "is_cloud", "iot_connected"]:
        if key not in node:
            continue
        ret[key] = node[key]
    ret["children"] = []
    for child in node["children"]:
        ret["children"].append(pprint_network(child))
    return ret


NETWORK = None


# NETWORK = {
#     "name": "market",
#     "flavor": TIER_1_FLAVOR,
#     "children": [
#         {
#             "name": "node_1",
#             "flavor": TIER_3_FLAVOR,
#             "latency": 3,
#             "children": [
#                 {
#                     "name": "node_2",
#                     "flavor": TIER_3_FLAVOR,
#                     "latency": 6,
#                     "children": [
#                         {
#                             "name": "node_3",
#                             "flavor": TIER_4_FLAVOR,
#                             "latency": 10,
#                             "children": [],
#                             "iot_connected": 0,
#                         },
#                         {
#                             "name": "node_34",
#                             "flavor": TIER_4_FLAVOR,
#                             "latency": 5,
#                             "children": [],
#                             "iot_connected": 0,
#                         },
#                     ],
#                 }
#             ],
#         },
#     ],
# }


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
    ret[node["name"]] = [(child["name"], child["latency"]) for child in children]
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


def adjacency_undirected(node):
    ret = defaultdict(lambda: [])

    def fun(node):
        children = node["children"] if "children" in node else []
        for child in children:
            ret[node["name"]] += [(child["name"], child["latency"])]
            ret[child["name"]] += [(node["name"], child["latency"])]
            fun(child)

    fun(node)
    return ret


def gen_net(nodes, callback):
    adjacency = adjacency_undirected(nodes)

    for name, latency in IOT_CONNECTION:
        # adjacency[name].append(("iot_emulation", latency))
        adjacency["iot_emulation"].append((name, latency))
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
        pq: List[Any] = []
        heapq.heappush(pq, (0, src))

        # Create a vector for distances and initialize all
        # distances as infinite (INF)
        dist: Dict[str, float] = defaultdict(lambda: float("inf"))
        dist[src] = 0

        while pq:
            # The first vertex in pair is the minimum distance
            # vertex, extract it from priority queue.
            # vertex label is stored in second of pair
            d, u = heapq.heappop(pq)

            # 'i' is used to get all adjacent vertices of a
            # vertex
            for v, latency in adjacency[u]:
                # If there is shorted path to v through u.
                if dist[v] > dist[u] + latency:
                    # Updating distance of v
                    dist[v] = dist[u] + latency
                    heapq.heappush(pq, (dist[v], v))

        return dist

    for node_name in adjacency.keys():
        latencies = dijkstra(node_name)  # modifies subtree_cumul
        for destination in latencies.keys():
            latency = latencies[destination]
            # print(f"{node_name} -> {destination} = {latency}")
            callback(node_name, destination, latency)


LOAD_NETWORK_FILE = "LOAD_NETWORK_FILE"
SAVE_NETWORK_FILE = "SAVE_NETWORK_FILE"

if NETWORK is None:
    save_network_file = os.getenv(SAVE_NETWORK_FILE)
    load_network_file = os.getenv(LOAD_NETWORK_FILE)
    if save_network_file and load_network_file:
        raise Exception(
            f"{SAVE_NETWORK_FILE} and {LOAD_NETWORK_FILE} env var should not be set together"
        )
    elif save_network_file and not load_network_file:
        NETWORK = network_generation()
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

if __name__ == "__main__":
    pprint.pprint(pprint_network(NETWORK), sort_dicts=False)
    print("Number of nodes:", len(FOG_NODES))
