import base64
import logging
import math
import os
import signal
import subprocess
import tempfile
from time import sleep
import uuid
from collections import defaultdict
from pathlib import Path

import click
import enoslib as en

# Enable rich logging
from enoslib import enostask
from enoslib.api import STATUS_FAILED, STATUS_OK, actions
from enoslib.errors import EnosFailedHostsError
from grid5000 import Grid5000
from grid5000.cli import auth

from monitoring import monitoring as mon

log = logging.getLogger("rich")

KUBECONFIG_LOCATION_K3S = "/etc/rancher/k3s/k3s.yaml"

TELEGRAF_IMAGE = "ghcr.io/volodiapg/telegraf:latest"
PROMETHEUS_IMAGE = "ghcr.io/volodiapg/prometheus:latest"
GRAFANA_IMAGE = "ghcr.io/volodiapg/grafana:latest"

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
    - name: proxied-fog-node-3030
      port: 3030
      targetPort: 3030
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
          value: "3030"
        - name: ROCKET_ADDRESS
          value: "0.0.0.0"
        - name: CONFIG
          value: "{conf}"
        ports:
        - containerPort: 3030
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
    - name: proxied-market-8000
      port: 8000
      targetPort: 8000
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
        - containerPort: 8000
        env:
        - name: ROCKET_ADDRESS
          value: "0.0.0.0"
"""

MARKET_CONNECTED_NODE = """MarketConnected (
    market_ip: "{market_ip}",
    market_port: 8000,
    my_id: "{my_id}",
    my_public_ip: "{my_public_ip}",
    my_public_port: 3030,
    tags: ["node_to_market", "{name}"],
)

"""

NODE_CONNECTED_NODE = """NodeConnected (
    parent_id: "{parent_id}",
    parent_node_ip: "{parent_ip}",
    parent_node_port: 3030,
    my_id: "{my_id}",
    my_public_ip: "{my_public_ip}",
    my_public_port: 3030,
    tags: ["node_to_node", "{name}"],
)

"""

# NETWORK = {
#     "name": "market",
#     "flavor": {"core": 10, "mem": 1024 * 16},
#     "children": [
#         {
#             "name": "caveirac",
#             "flavor": {"core": 4, "mem": 1024 * 4},
#             "latency": 150
#         },
#         {
#             "name": "brest",
#             "flavor": {"core": 2, "mem": 1024 * 2},
#             "latency": 50
#         },
#     ]
# }

TIER_3_FLAVOR = {"core": 2, "mem": 1024 * 4}
# TIER_3_FLAVOR = {"core": 4, "mem": 1024 * 4}
# TIER_3_FLAVOR = {"core": 2, "mem": 1024 * 2}

NETWORK = {
    "name": "market",
    "flavor": TIER_3_FLAVOR,#{"core": 10, "mem": 1024 * 16},
    "children": [
        {
            "name": "paris",
            "flavor": TIER_3_FLAVOR,
            "latency": 50,
            "children": [
                {
                    "name": "rennes",
                    "flavor": TIER_3_FLAVOR,
                    "latency": 150,
                    "children": [
                        {
                            "name": "rennes-50",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 50,
                            "children": [
                                {
                                    "name": "st-greg-50",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 50,
                                },
                                {
                                    "name": "st-greg-75",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 75,
                                },
                            ],
                        },
                        {
                            "name": "rennes-75",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 75,
                            "children": [
                                {
                                    "name": "cesson-50",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 50,
                                },
                                {
                                    "name": "cesson-75",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 75,
                                },
                            ],
                        },
                    ],
                },
                {
                    "name": "nantes",
                    "flavor": TIER_3_FLAVOR,
                    "latency": 100,
                    "children": [
                        {
                            "name": "nantes-50",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 50,
                            "children": [
                                {
                                    "name": "clisson-50",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 50,
                                },
                                {
                                    "name": "clisson-75",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 75,
                                },
                            ],
                        },
                        {
                            "name": "nantes-75",
                            "flavor": TIER_3_FLAVOR,
                            "latency": 75,
                            "children": [
                                {
                                    "name": "cholet-50",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 50,
                                },
                                {
                                    "name": "cholet-75",
                                    "flavor": TIER_3_FLAVOR,
                                    "latency": 75,
                                },
                            ],
                        },
                    ],
                },
            ],
        }
    ],
}

# Remove a unit so that the hosts are not saturated
NB_CPU_PER_MACHINE_PER_CLUSTER = {
    "gros": {"core": 18 - 1, "mem": 1024 * (96 - 2)},
    "paravance": {"core": 2 * 8 - 1, "mem": 1024 * (128 - 2)},
    "dahu": {"core": 2 * 16 - 1, "mem": 1024 * (192 - 2)},
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


def get_aliases(env):
    roles = env["roles"]
    ret = {}
    for node in FOG_NODES:
        role = roles[node]
        alias = role[0].alias
        ret[alias] = node
    ret["market"] = roles["market"][0].alias

    return ret


def get_aliases_from_ip(env):
    roles = env["roles"]
    ret = {}
    for node in FOG_NODES:
        role = roles[node]
        alias = role[0].address + ":3030"
        ret[alias] = node
    ret[roles["market"][0].address + ":3030"] = "market"

    return ret


def log_cmd(env, results):
    if results.filter(status=STATUS_FAILED):
        for data in results.filter(status=STATUS_FAILED).data:
            data = data.payload
            if data["stdout"]:
                log.error(data["stdout"])
            if data["stderr"]:
                log.error(data["stderr"])

    if results.filter(status=STATUS_OK):
        for data in results.filter(status=STATUS_OK).data:
            host = data.host
            data = data.payload
            if data["stdout"]:
                print(data["stdout"])
                try:
                    with tempfile.NamedTemporaryFile(
                        dir="/tmp", delete=False
                    ) as tmpfile:
                        with open(tmpfile.name, "w") as file:
                            file.write(data["stdout"])
                        alias_name = get_aliases(env).get(host, host)
                        subprocess.run(
                            [
                                "mprocs",
                                "--server",
                                "127.0.0.1:4050",
                                "--ctl",
                                f'{{c: add-proc, cmd: "echo {alias_name} && cat {tmpfile.name} | tail -n 900 && sleep 1"}}',
                            ]
                        )
                except:
                    log.warning("Cannot use mprocs to output nice things organized.")
            if data["stderr"]:
                log.error(data["stderr"])


def open_tunnel(address, port, rest_of_url=""):
    tunnel = en.G5kTunnel(address=address, port=port)
    local_address, local_port, _ = tunnel.start()
    print(f"tunnel opened: {port} -> http://localhost:{local_port}{rest_of_url}")
    return local_address, local_port


@click.group()
def cli(**kwargs):
    """Experiment with k3s in G5K.

    Don't forget to clean with the `clean` verb.

    P.S.
    Errors with ssh may arise, consider `ln -s ~/.ssh/id_ed25519.pub ~/.ssh/id_rsa.pub` if necessary.
    """
    en.init_logging()


@cli.command()
@click.option("--g5k_user", required=True, help="G5K username")
@click.option("--force", is_flag=True, help="force overwrite")
def init(g5k_user, force):
    """Initialize the grid5000 connection options."""
    conf_file = Path.home() / ".python-grid5000.yaml"

    if not conf_file.exists() or force:
        # will prompt for the password and write the authentication file
        auth(g5k_user)

        conf_file.chmod(0o600)

    _ = Grid5000.from_yaml(conf_file)

    en.check()


def gen_vm_conf(node):
    ret = defaultdict(lambda: [])
    children = node["children"] if "children" in node else []
    for child in children:
        ret[frozenset(child["flavor"].items())].append(child["name"])
        for key, value in gen_vm_conf(child).items():
            for val in value:
                ret[key].append(val)

    return ret


def assign_vm_to_hosts(node, conf, cluster, nb_cpu_per_host, mem_total_per_host):
    attributions = {}
    vms = gen_vm_conf(node)
    for key, value in vms.items():
        flavor = {x: y for (x, y) in key}
        core = flavor["core"]
        mem = flavor["mem"]

        core_used = 0
        mem_used = 0
        vm_id = str(uuid.uuid4())
        nb_vms = 0
        for vm_name in value:
            core_used += core
            mem_used += mem

            if core_used > nb_cpu_per_host or mem_used > mem_total_per_host:
                if nb_vms == 0:
                    raise Exception(
                        "The VM requires more resources than the node can provide"
                    )

                conf.add_machine(
                    roles=["master", "fog_node", "prom_agent", vm_id],
                    cluster=cluster,
                    number=nb_vms,
                    flavour_desc=flavor,
                )
                core_used = 0
                mem_used = 0
                nb_vms = 0
                vm_id = str(uuid.uuid4())

            nb_vms += 1
            attributions[vm_name] = vm_id

        # Still an assignation left?
        if nb_vms > 0:
            conf.add_machine(
                roles=["master", "fog_node", "prom_agent", vm_id],
                cluster=cluster,
                number=nb_vms,
                flavour_desc=flavor,
            )

    return attributions


def attributes_roles(vm_attributions, roles):
    count = defaultdict(lambda: 0)
    for vm, instance_id in vm_attributions.items():
        roles[vm] = [roles[instance_id][count[instance_id]]]
        count[instance_id] += 1


@cli.command()
@click.option("--force", is_flag=True, help="destroy and up")
@enostask(new=True)
def up(force, env=None, **kwargs):
    """Claim the resources and setup k3s."""
    env["CLUSTER"] = os.environ["CLUSTER"]
    cluster = env["CLUSTER"]

    if cluster not in NB_CPU_PER_MACHINE_PER_CLUSTER:
        print(
            f"Consider adding support for {cluster} in the variable NB_CPU_PER_MACHINE_PER_CLUSTER (I need more "
            f"details about this cluster to support it"
        )
        exit(126)

    nb_cpu_per_machine = NB_CPU_PER_MACHINE_PER_CLUSTER[cluster]["core"]
    mem_per_machine = NB_CPU_PER_MACHINE_PER_CLUSTER[cluster]["mem"]

    print(f"Deploying on {cluster}")

    conf = (
        en.VMonG5kConf.from_settings(job_name="En0SLib FTW ❤️", walltime="1:00:00")
        .add_machine(
            roles=["master", "market", "prom_agent"],
            cluster=cluster,
            number=1,
            flavour_desc=NETWORK["flavor"],
        )
        .add_machine(roles=["prom_master"], cluster=cluster, number=1, flavour="large")
        .add_machine(
            roles=["prom_agent", "iot_emulation"],
            cluster=cluster,
            number=1,
            flavour="large",
        )
    )

    assignations = assign_vm_to_hosts(
        NETWORK, conf, cluster, nb_cpu_per_machine, mem_per_machine
    )

    print(
        f"I need {len(conf.machines)} bare-metal nodes in total, running a total of {len(assignations)} Fog node VMs"
    )

    conf.finalize()

    provider = en.VMonG5k(conf)

    roles, networks = provider.init(force_deploy=force)

    en.wait_for(roles)

    roles = en.sync_info(roles, networks)

    attributes_roles(assignations, roles)

    roles = en.sync_info(roles, networks)

    env["provider"] = provider
    env["roles"] = roles
    env["networks"] = networks

    netem = en.NetemHTB()

    env["netem"] = netem

    establish_netem(env)

    k3s = en.K3s(master=roles["master"], agent=list())

    k3s.deploy()

    with actions(roles=roles["master"], gather_facts=False) as p:
        p.shell(
            ("curl -sSL https://cli.openfaas.com | sudo -E sh"),
            task_name="[master] Installing OpenFaaS CLI",
        )
        p.shell(
            ("curl -SLsf https://dl.get-arkade.dev/ | sudo -E sh"),
            task_name="[master] Installing Arkade",
        )
        p.shell(
            (
                f"export KUBECONFIG={KUBECONFIG_LOCATION_K3S} && sudo -E arkade install openfaas"
            ),
            task_name="[master] Installing OpenFaaS",
        )
        # p.shell(
        #     f"export KUBECONFIG={KUBECONFIG_LOCATION_K3S} && kubectl -n kubernetes-dashboard describe secret admin-user-token | grep '^token'",
        #     task_name="token",
        # )
        p.shell(
            f"k3s kubectl port-forward -n openfaas svc/gateway 8080:8080",
            background=True,
        )
    # env["k3s-token"] = [res.stdout for res in p.results.filter(task="token")]

    # Deploy the echo node
    with en.Docker(agent=roles["iot_emulation"]):
        with actions(roles=roles["iot_emulation"]) as p:
            p.shell(
                "docker run -p 3030:3030 ghcr.io/volodiapg/iot_emulation:latest",
                task_name="Run iot_emulation on the endpoints",
                background=True,
            )


def establish_netem(env):
    netem = env["netem"]
    roles = env["roles"]

     # generate the network
    gen_net(NETWORK, netem, roles)

    # Connect the extremities to the echo server
    for extremity in EXTREMITIES:
        netem.add_constraints(
            src=roles[extremity],
            dest=roles["iot_emulation"],
            delay="0ms",
            rate="1gbit",
            symetric=True,
        )

    netem.deploy()
    # netem.validate()

def drop_netem(env):
    netem = env["netem"]
    netem.destroy()

def gen_net(node, netem, roles):
    children = node["children"] if "children" in node else []

    for child in children:
        print(
            f"Setting lat of {child['latency']} btw {node['name']} and {child['name']}"
        )
        netem.add_constraints(
            src=roles[node["name"]],
            dest=roles[child["name"]],
            delay=str(child["latency"]) + "ms",
            rate="1gbit",
            symetric=True,
        )
        gen_net(child, netem, roles)


@cli.command()
@enostask()
def monitoring(env=None, **kwargs):
    """Remove the constraints on the network links"""
    roles = env["roles"]
    drop_netem(env)
    monitor = mon.TPGMonitoring(
        collector=roles["prom_master"][0],
        agent=roles["prom_agent"],
        ui=roles["prom_master"][0],
        telegraf_image=TELEGRAF_IMAGE,
        prometheus_image=PROMETHEUS_IMAGE,
        grafana_image=GRAFANA_IMAGE,
    )
    monitor.deploy()
    env["monitor"] = monitor
    establish_netem(env)


@cli.command()
@enostask()
def k3s_config(env=None, **kwargs):
    """SCP the remote kubeconfig files"""
    for out in env["k3s-token"]:
        print(out)


@enostask()
def aliases(env=None, **kwargs):
    """Get aliases"""
    return get_aliases_from_ip(env)


def gen_conf(node, parent_id, parent_ip, ids):
    (my_id, my_ip) = ids[node["name"]]
    conf = NODE_CONNECTED_NODE.format(
        parent_id=parent_id,
        parent_ip=parent_ip,
        my_id=my_id,
        my_public_ip=my_ip,
        name=node["name"],
    )

    children = node["children"] if "children" in node else []

    return [
        (node["name"], conf),
        *[gen_conf(node, my_id, my_ip, ids) for node in children],
    ]


@cli.command()
@enostask()
def k3s_deploy(env=None, **kwargs):
    roles = env["roles"]
    drop_netem(env)

    en.run_command(
        "k3s kubectl delete -f /tmp/node_conf.yaml || true",
        roles=roles["master"],
        task_name="Removing existing fog_node software",
    )

    en.run_command(
        "k3s kubectl delete -f /tmp/market.yaml || true",
        roles=roles["master"],
        task_name="Removing existing market software",
    )

    ids = {
        node_name: (uuid.uuid4(), roles[node_name][0].address)
        for node_name in FOG_NODES
    }
    market_id = uuid.uuid4()
    market_ip = roles[NETWORK["name"]][0].address
    confs = [
        (
            NETWORK["name"],
            MARKET_CONNECTED_NODE.format(
                market_ip=market_ip,
                my_id=market_id,
                my_public_ip=market_ip,
                name="cloud",
            ),
        )
    ]
    confs = list(
        flatten(
            [
                *confs,
                *[
                    gen_conf(child, market_id, market_ip, ids)
                    for child in NETWORK["children"]
                ],
            ]
        )
    )

    for (name, conf) in confs:
        deployment = FOG_NODE_DEPLOYMENT.format(
            conf=base64.b64encode(bytes(conf, "utf-8")).decode("utf-8")
        )
        roles[name][0].set_extra(fog_node_deployment=deployment)

    roles[NETWORK["name"]][0].set_extra(market_deployment=MARKET_DEPLOYMENT)

    try:
        res = en.run_command(
            "cat << EOF > /tmp/node_conf.yaml\n"
            "{{ fog_node_deployment }}\n"
            "EOF\n"
            "k3s kubectl create -f /tmp/node_conf.yaml",
            roles=roles["master"],
            task_name="Deploying fog_node software",
        )
        log_cmd(env, res)
    except EnosFailedHostsError as err:
        for host in err.hosts:
            payload = host.payload
            if payload["stdout"]:
                print(payload["stdout"])
            if payload["stderr"]:
                log.error(payload["stderr"])

    try:
        res = en.run_command(
            "cat << EOF > /tmp/market.yaml\n"
            "{{ market_deployment }}\n"
            "EOF\n"
            "k3s kubectl create -f /tmp/market.yaml",
            roles=roles["market"],
            task_name="Deploying market software",
        )
        log_cmd(env, res)
    except EnosFailedHostsError as err:
        for host in err.hosts:
            payload = host.payload
            if payload["stdout"]:
                print(payload["stdout"])
            if payload["stderr"]:
                log.error(payload["stderr"])

    establish_netem(env)


@cli.command()
@click.option("--all", is_flag=True, help="all namespaces")
@enostask()
def health(env=None, all=False, **kwargs):
    roles = env["roles"]

    command = "kubectl get deployments -n openfaas"
    if all:
        command = "kubectl get deployments --all-namespaces"
    res = en.run_command(command, roles=roles["master"])
    log_cmd(env, res)


@cli.command()
@enostask()
def functions(env=None, **kwargs):
    roles = env["roles"]
    res = en.run_command(
        "kubectl get deployments -n openfaas-fn", roles=roles["master"]
    )
    log_cmd(env, res)


@cli.command()
@enostask()
def toto(env=None, **kwargs):
    roles = env["roles"]
    res = en.run_command(
        "kubectl logs deployment/primes -n openfaas-fn | tail -n 1000",
        roles=roles["master"],
    )
    log_cmd(env, res)


@cli.command()
@click.option("--all", is_flag=True, help="all namespaces")
@enostask()
def logs(env=None, all=False, **kwargs):
    roles = env["roles"]
    if all:
        res = en.run_command(
            "kubectl logs deployment/fog-node -n openfaas", roles=roles["master"]
        )
        log_cmd(env, res)

    res = en.run_command(
        "kubectl logs deployment/market -n openfaas", roles=roles["market"]
    )
    log_cmd(env, res)


@cli.command()
@click.option("--file", required=False, help="Write output to file")
@enostask()
def openfaas_login(env=None, file=None, **kwargs):
    """Get OpenFaaS login info.

    Username is always `admin`.
    """
    roles = env["roles"]
    res = en.run_command(
        'echo -n $(kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo)',
        roles=roles["master"],
    )
    log_cmd(env, res)
    if file:
        with open(file, "w") as f:
            f.write(str(res.filter(status=STATUS_OK).data[0].payload["stdout"]) + "\n")


@cli.command()
@click.option("--all", required=False, is_flag=True, help="Also tunnel fog nodes")
@enostask()
def tunnels(env=None, all=False, **kwargs):
    """Open the tunnels to the K8S UI and to OpenFaaS from the current host."""
    procs = []
    try:
        roles = env["roles"]
        if all:
            for role in roles["master"]:
                address = role.address
                open_tunnel(address, 31112)  # OpenFaas
                open_tunnel(address, 3030)  # Fog Node
                open_tunnel(
                    address,
                    8001,
                    "/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/#/node?namespace=default",
                )  # K8S API

        tun = {}  # out_port, (res of tunnels())
        flag = False
        for role in roles["market"]:
            res = open_tunnel(role.address, 8000)  # Market
            if flag == False:
                tun[8080] = res
                flag = True

        tun[9090] = open_tunnel(env["roles"]["prom_master"][0].address, 9090)
        tun[7070] = open_tunnel(env["monitor"].ui.address, 3000)
        tun[3030] = open_tunnel(env["roles"]["iot_emulation"][0].address, 3030)

        # The os.setsid() is passed in the argument preexec_fn so
        # it's run after the fork() and before  exec() to run the shell.

        for port, (_, local_port) in tun.items():
            cmd = f"ssh -N -L {port}:127.0.0.1:{local_port} -i $HOME/.ssh/id_rsa.pub 127.0.0.1"
            pro = subprocess.Popen(
                cmd, stdout=subprocess.PIPE, shell=True, preexec_fn=os.setsid
            )
            print(f"{cmd}, aka new tunnel: {local_port} -> {port}")
            procs.append(pro)
        sleep(1)
        print("Press Enter to kill.")
        input()
    finally:
        for pro in procs:
            os.killpg(
                os.getpgid(pro.pid), signal.SIGTERM
            )  # Send the signal to all the process groups


@cli.command()
@enostask()
def endpoints(env=None, **kwargs):
    """List the address of the end-nodes in the Fog network"""
    roles = env["roles"]
    for extremity in EXTREMITIES:
        role = roles[extremity][0]
        address = role.address
        print(f"{extremity} -> {address}")

    print(f"---\nIot emulation IP -> {roles['iot_emulation'][0].address}")


@cli.command()
@click.option("--src", required=True, help="Source of the latency request")
@click.option("--dest", required=True, help="Dest of the latency request")
@enostask()
def latency(src, dest, env=None):
    roles = env["roles"]
    src = roles[src][0]
    dest = roles[dest][0]

    res = en.run_command(
        f"""curl -w @- -o /dev/null -X HEAD -s "http://{dest.address}:3030/api/health" <<'EOF'
   time_namelookup:  %{{time_namelookup}}s\\n
      time_connect:  %{{time_connect}}s\\n
   time_appconnect:  %{{time_appconnect}}s\\n
  time_pretransfer:  %{{time_pretransfer}}s\\n
     time_redirect:  %{{time_redirect}}s\\n
time_starttransfer:  %{{time_starttransfer}}s\\n
---\\n
        time_total:  %{{time_total}}s\\n
EOF""",
        roles=[src],
    )
    log_cmd(env, res)


@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env["provider"]

    provider.destroy()


if __name__ == "__main__":
    cli()
