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
  type: LoadBalancer
  ports:
    - name: proxied-fog-node-3003
      port: 3003
      targetPort: 3003
      protocol: TCP
    - name: proxied-fog-node-3004
      port: 3004
      targetPort: 3004
      protocol: TCP
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
        image: ghcr.io/volodiapg/fog_node:latest
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
          value: "8080"
        - name: ROCKET_PORT
          value: "3003"
        - name: ROCKET_ADDRESS
          value: "0.0.0.0"
        - name: CONFIG
          value: "{conf}"
        - name: LOG_CONFIG_PATH
          value: "/var/log"
        - name: LOG_CONFIG_FILENAME
          value: "stdout.log"
        - name: RUST_LOG
          value: "warn,fog_node=trace,openfaas=trace,kube_metrics=trace,helper=trace"
        ports:
        - containerPort: 3003
        - containerPort: 3004
        volumeMounts:
        - name: log-storage-fog-node
          mountPath: /var/log
      - name: sidecar-logs
        image: ghcr.io/volodiapg/busybox:latest
        args: [/bin/sh, -c, 'tail -n+1 -F /mnt/log/stdout.log']
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
  type: LoadBalancer
  ports:
    - name: proxied-market-3008
      port: 3008
      targetPort: 3008
      protocol: TCP
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
        image: ghcr.io/volodiapg/market:latest
        ports:
        - containerPort: 3008
        env:
        - name: ROCKET_ADDRESS
          value: "0.0.0.0"
        - name: ROCKET_PORT
          value: "3008"
        volumeMounts:
        - name: log-storage-market
          mountPath: /var/log
      - name: sidecar-logs
        image: ghcr.io/volodiapg/busybox:latest
        args: [/bin/sh, -c, 'tail -n+1 -F /mnt/log/stdout.log']
        volumeMounts:
        - name: log-storage-market
          readOnly: true
          mountPath: /mnt/log
      volumes:
      - name: log-storage-market
        emptyDir: {}
"""

MARKET_CONNECTED_NODE = """MarketConnected (
    market_ip: "{market_ip}",
    market_port: "3008",
    my_id: "{my_id}",
    my_public_ip: "{my_public_ip}",
    my_public_port_http: "3003",
    my_public_port_rpc: "3004",
    tags: ["node_to_market", "{name}"],
)

"""

NODE_CONNECTED_NODE = """NodeConnected (
    parent_id: "{parent_id}",
    parent_node_ip: "{parent_ip}",
    parent_node_port_http: "3003",
    parent_node_port_rpc: "3004",
    my_id: "{my_id}",
    my_public_ip: "{my_public_ip}",
    my_public_port_http: "3003",
    my_public_port_rpc: "3004",
    tags: ["node_to_node", "{name}"],
)

"""

TIER_3_FLAVOR = {"core": 2, "mem": 1024 * 4}
TIER_2_FLAVOR = {"core": 4, "mem": 1024 * 8}
TIER_1_FLAVOR = {"core": 8, "mem": 1024 * 16}

NETWORK = {
    "name": "market",
    "flavor": TIER_1_FLAVOR,
    "children": [
        {
            "name": "paris",
            "flavor": TIER_2_FLAVOR,
            "latency": 5,
            "children": [
                {
                    "name": "rennes",
                    "flavor": TIER_2_FLAVOR,
                    "latency": 25,
                    "children": [
                        {
                            "name": "st-greg-5",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 5,
                        },
                        {
                            "name": "st-greg-10",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 10,
                        },
                    ],
                }
            ],
        },
    ],
}
# NETWORK = {
#     "name": "market",
#     "flavor": TIER_1_FLAVOR,#{"core": 10, "mem": 1024 * 16},
#     "children": [
#         {
#             "name": "paris",
#             "flavor": TIER_1_FLAVOR,
#             "latency": 50,
#             "children": [
#                 {
#                     "name": "rennes",
#                     "flavor": TIER_2_FLAVOR,
#                     "latency": 150,
#                     "children": [
#                         {
#                             "name": "rennes-50",
#                             "flavor": TIER_3_FLAVOR,
#                             "latency": 50,
#                             "children": [
#                                 {
#                                     "name": "st-greg-50",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 50,
#                                 },
#                                 {
#                                     "name": "st-greg-75",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 75,
#                                 },
#                             ],
#                         },
#                         {
#                             "name": "rennes-75",
#                             "flavor": TIER_3_FLAVOR,
#                             "latency": 75,
#                             "children": [
#                                 {
#                                     "name": "cesson-50",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 50,
#                                 },
#                                 {
#                                     "name": "cesson-75",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 75,
#                                 },
#                             ],
#                         },
#                     ],
#                 },
#                 {
#                     "name": "nantes",
#                     "flavor": TIER_3_FLAVOR,
#                     "latency": 100,
#                     "children": [
#                         {
#                             "name": "nantes-50",
#                             "flavor": TIER_3_FLAVOR,
#                             "latency": 50,
#                             "children": [
#                                 {
#                                     "name": "clisson-50",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 50,
#                                 },
#                                 {
#                                     "name": "clisson-75",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 75,
#                                 },
#                             ],
#                         },
#                         {
#                             "name": "nantes-75",
#                             "flavor": TIER_3_FLAVOR,
#                             "latency": 75,
#                             "children": [
#                                 {
#                                     "name": "cholet-50",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 50,
#                                 },
#                                 {
#                                     "name": "cholet-75",
#                                     "flavor": TIER_3_FLAVOR,
#                                     "latency": 75,
#                                 },
#                             ],
#                         },
#                     ],
#                 },
#             ],
#         }
#     ],
# }

# Remove a unit so that the hosts are not saturated
NB_CPU_PER_MACHINE_PER_CLUSTER = {
    "gros": {"core": 18 - 2, "mem": 1024 * (96 - 4)},
    "paravance": {"core": 2 * 8 - 2, "mem": 1024 * (128 - 4)},
    "dahu": {"core": 2 * 16 - 2, "mem": 1024 * (192 - 4)},
}


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


def adjacency(node):
    children = node["children"] if "children" in node else []
    ret = {}
    ret[node["name"]] = [(child["name"], child["latency"]) for child in children]
    for child in children:
        ret = {**ret, **adjacency(child)}

    return ret


FOG_NODES = list(flatten([gen_fog_nodes_names(child) for child in NETWORK["children"]]))
EXTREMITIES = list(
    flatten([get_extremities_name(child) for child in NETWORK["children"]])
)
ADJACENCY = adjacency(NETWORK)
