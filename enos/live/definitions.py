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
        - name: CONFIG
          value: "{conf}"
        - name: LOG_CONFIG_PATH
          value: "/var/log"
        - name: LOG_CONFIG_FILENAME
          value: "{node_name}.log"
        - name: RUST_LOG
          value: "warn,fog_node=info,openfaas=info,kube_metrics=info,helper=info"
        - name: COLLECTOR_IP
          value: "{collector_ip}"
        - name: COLLECTOR_PORT
          value: "14268"
        ports:
        - containerPort: 3003
        - containerPort: 3004
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
          value: "3008"
        - name: COLLECTOR_IP
          value: "{collector_ip}"
        - name: COLLECTOR_PORT
          value: "14268"
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

TIER_3_FLAVOR = {"core": 4, "mem": 1024 * 4}
TIER_2_FLAVOR = {"core": 8, "mem": 1024 * 8}
TIER_1_FLAVOR = {"core": 14, "mem": 1024 * 16}

NETWORK = {
    "name": "market",
    "flavor": TIER_1_FLAVOR,
    "children": [
        {
            "name": "paris",
            "flavor": TIER_1_FLAVOR,
            "latency": 30,
            "children": [
                {
                    "name": "nantes",
                    "flavor": TIER_2_FLAVOR,
                    "latency": 25,
                },
                {
                    "name": "rennes",
                    "flavor": TIER_2_FLAVOR,
                    "latency": 20,
                    "children": [
                        {
                            "name": "st-greg",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 10,
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
                                {
                                    "name": "st-greg-1",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 1,
                                },
                            ],
                        },
                        {
                            "name": "cesson",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 7,
                            "children": [
                                {
                                    "name": "cesson-5",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 5,
                                },
                                {
                                    "name": "cesson-10",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 10,
                                },
                                {
                                    "name": "cesson-1",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 1,
                                },
                            ],
                        },
                    ],
                },
            ],
        },
    ],
}

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
