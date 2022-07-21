import base64
import logging
import os
import subprocess
import tempfile
import uuid
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
#     ]
# }

NETWORK = {
    "name": "market",
    "flavor": {"core": 10, "mem": 1024 * 16},
    "children": [
        {
            "name": "london",
            "flavor": {"core": 4, "mem": 1024 * 4},
            "latency": 150,
            "children": [
                {
                    "name": "berlin",
                    "flavor": {"core": 2, "mem": 1024 * 2},
                    "latency": 100
                }
            ]
        },
        {
            "name": "rennes",
            "flavor": {"core": 6, "mem": 1024 * 4},
            "latency": 100,
            "children": [
                {
                    "name": "vannes",
                    "flavor": {"core": 2, "mem": 1024 * 2},
                    "latency": 100,
                    "children": [
                        {
                            "name": "brest",
                            "flavor": {"core": 2, "mem": 1024 * 2},
                            "latency": 50
                        },
                        {
                            "name": "caveirac",
                            "flavor": {"core": 4, "mem": 1024 * 4},
                            "latency": 100
                        }
                    ]
                },
                {
                    "name": "nantes",
                    "flavor": {"core": 4, "mem": 1024 * 4},
                    "latency": 250
                }
            ]
        }
    ]
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
    ret[node['name']] = [(child['name'], child['latency']) for child in children]
    for child in children:
        ret = {**ret, **adjacency(child)}

    return ret


FOG_NODES = list(flatten([gen_fog_nodes_names(child) for child in NETWORK["children"]]))
EXTREMITIES = list(flatten([get_extremities_name(child) for child in NETWORK["children"]]))
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
            if data['stdout']:
                log.error(data['stdout'])
            if data['stderr']:
                log.error(data['stderr'])

    if results.filter(status=STATUS_OK):
        for data in results.filter(status=STATUS_OK).data:
            host = data.host
            data = data.payload
            if data['stdout']:
                print(data['stdout'])
                try:
                    with tempfile.NamedTemporaryFile(dir="/tmp", delete=False) as tmpfile:
                        with open(tmpfile.name, "w") as file:
                            file.write(data["stdout"])
                        alias_name = get_aliases(env).get(host, host)
                        subprocess.run(["mprocs", "--server", "127.0.0.1:4050", "--ctl",
                                        f'{{c: add-proc, cmd: "echo {alias_name} && cat {tmpfile.name}"}}'])
                except:
                    log.warning("Cannot use mprocs to output nice things organized.")
            if data['stderr']:
                log.error(data['stderr'])


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
def init():
    """Initialize the grid5000 connection options."""
    conf_file = Path.home() / ".python-grid5000.yaml"

    if not conf_file.exists():
        # CHANGE ME!
        g5k_user = "voparolguarino"
        # will prompt for the password and write the authentication file
        auth(g5k_user)

        conf_file.chmod(0o600)

    _ = Grid5000.from_yaml(conf_file)

    en.check()


def gen_vm_conf(node, conf, cluster):
    children = node["children"] if "children" in node else []
    for child in children:
        conf.add_machine(
            roles=["master", "fog_node", child["name"], "prom_agent"],
            cluster=cluster,
            number=1,
            flavour_desc=child["flavor"]
        )
        gen_vm_conf(child, conf, cluster)


@cli.command()
@click.option("--force", is_flag=True, help="destroy and up")
@enostask(new=True)
def up(force, env=None, **kwargs):
    """Claim the resources and setup k3s."""
    env["CLUSTER"] = os.environ["CLUSTER"]
    cluster=env["CLUSTER"]
    print(f"Deploying on {cluster}")

    conf = (
        en
        .VMonG5kConf
        .from_settings(job_name="En0SLib FTW ❤️", walltime="1:00:00")
        .add_machine(
            roles=["master", "market", "prom_agent"],
            cluster=cluster,
            number=1,
            flavour_desc=NETWORK["flavor"]
        )
        .add_machine(
            roles=["prom_master"],
            cluster=cluster,
            number=1,
            flavour="large"
        )
        .add_machine(
            roles=["prom_agent", "iot_emulation"],
            cluster=cluster,
            number=1,
            flavour="large"
        )
    )

    print(FOG_NODES)

    gen_vm_conf(NETWORK, conf, cluster)

    conf.finalize()

    provider = en.VMonG5k(conf)

    roles, networks = provider.init(force_deploy=force)

    env['provider'] = provider
    env['roles'] = roles
    env['networks'] = networks

    en.wait_for(roles)

    roles = en.sync_info(roles, networks)

    netem = en.NetemHTB()

    env['netem'] = netem

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
    netem.validate()

    k3s = en.K3s(master=roles['master'], agent=list())

    k3s.deploy()

    with actions(roles=roles['master'], gather_facts=False) as p:
        p.shell(
            ("curl -sSL https://cli.openfaas.com | sudo -E sh"),
            task_name="[master] Installing OpenFaaS CLI"
        )
        p.shell(
            ("curl -SLsf https://dl.get-arkade.dev/ | sudo -E sh"),
            task_name="[master] Installing Arkade"
        )
        p.shell(
            (f"export KUBECONFIG={KUBECONFIG_LOCATION_K3S} && sudo -E arkade install openfaas"),
            task_name="[master] Installing OpenFaaS"
        )
        p.shell(
            f"export KUBECONFIG={KUBECONFIG_LOCATION_K3S} && kubectl -n kubernetes-dashboard describe secret admin-user-token | grep '^token'",
            task_name="token",
        )
        p.shell(f"k3s kubectl port-forward -n openfaas svc/gateway 8080:8080", background=True)
    env["k3s-token"] = [res.stdout for res in p.results.filter(task="token")]

    # Deploy the echo node
    with en.Docker(agent=roles["iot_emulation"]):
        with actions(roles=roles["iot_emulation"]) as p:
            p.shell('docker run -p 7070:7070 ghcr.io/volodiapg/iot_emulation:latest',
                    task_name="Run iot_emulation on the endpoints", background=True)


def gen_net(node, netem, roles):
    children = node["children"] if "children" in node else []

    for child in children:
        print(f"Setting lat of {child['latency']} btw {node['name']} and {child['name']}")
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
    roles = env['roles']
    monitor = mon.TPGMonitoring(collector=roles["prom_master"][0], agent=roles["prom_agent"],
                                ui=roles["prom_master"][0], telegraf_image=TELEGRAF_IMAGE,
                                prometheus_image=PROMETHEUS_IMAGE, grafana_image=GRAFANA_IMAGE)
    monitor.deploy()
    env['monitor'] = monitor


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
    conf = NODE_CONNECTED_NODE.format(parent_id=parent_id, parent_ip=parent_ip, my_id=my_id, my_public_ip=my_ip,
                                      name=node["name"])

    children = node["children"] if "children" in node else []

    return [(node["name"], conf), *[gen_conf(node, my_id, my_ip, ids) for node in children]]


@cli.command()
@enostask()
def k3s_deploy(env=None, **kwargs):
    roles = env["roles"]

    en.run_command('k3s kubectl delete -f /tmp/node_conf.yaml || true',
                   roles=roles["master"],
                   task_name="Removing existing fog_node software")

    en.run_command('k3s kubectl delete -f /tmp/market.yaml || true',
                   roles=roles["master"],
                   task_name="Removing existing market software")

    ids = {node_name: (uuid.uuid4(), roles[node_name][0].address) for node_name in FOG_NODES}
    market_id = uuid.uuid4()
    market_ip = roles[NETWORK["name"]][0].address
    confs = [
        (NETWORK["name"],
         MARKET_CONNECTED_NODE.format(market_ip=market_ip, my_id=market_id, my_public_ip=market_ip, name="cloud"))]
    confs = list(flatten([*confs, *[gen_conf(child, market_id, market_ip, ids) for child in NETWORK["children"]]]))

    for (name, conf) in confs:
        deployment = FOG_NODE_DEPLOYMENT.format(conf=base64.b64encode(bytes(conf, "utf-8")).decode("utf-8"))
        roles[name][0].set_extra(fog_node_deployment=deployment)

    roles[NETWORK["name"]][0].set_extra(market_deployment=MARKET_DEPLOYMENT)

    try:
        res = en.run_command(
            'cat << EOF > /tmp/node_conf.yaml\n'
            '{{ fog_node_deployment }}\n'
            'EOF\n'
            'k3s kubectl create -f /tmp/node_conf.yaml',
            roles=roles["master"],
            task_name="Deploying fog_node software")
        log_cmd(env, res)
    except EnosFailedHostsError as err:
        for host in err.hosts:
            payload = host.payload
            if payload['stdout']:
                print(payload['stdout'])
            if payload['stderr']:
                log.error(payload['stderr'])

    try:
        res = en.run_command(
            'cat << EOF > /tmp/market.yaml\n'
            '{{ market_deployment }}\n'
            'EOF\n'
            'k3s kubectl create -f /tmp/market.yaml',
            roles=roles["market"],
            task_name="Deploying market software")
        log_cmd(env, res)
    except EnosFailedHostsError as err:
        for host in err.hosts:
            payload = host.payload
            if payload['stdout']:
                print(payload['stdout'])
            if payload['stderr']:
                log.error(payload['stderr'])


@cli.command()
@click.option("--all", is_flag=True, help="all namespaces")
@enostask()
def health(env=None, all=False, **kwargs):
    roles = env['roles']

    command = "kubectl get deployments -n openfaas"
    if all:
        command = "kubectl get deployments --all-namespaces"
    res = en.run_command(
        command,
        roles=roles["master"])
    log_cmd(env, res)


@cli.command()
@enostask()
def functions(env=None, **kwargs):
    roles = env['roles']
    res = en.run_command(
        'kubectl get deployments -n openfaas-fn',
        roles=roles["master"])
    log_cmd(env, res)


@cli.command()
@enostask()
def toto(env=None, **kwargs):
    roles = env['roles']
    res = en.run_command(
        'kubectl logs deployment/primes -n openfaas-fn',
        roles=roles["master"])
    log_cmd(env, res)


@cli.command()
@click.option("--all", is_flag=True, help="all namespaces")
@enostask()
def logs(env=None, all=False, **kwargs):
    roles = env['roles']
    if all:
        res = en.run_command(
            'kubectl logs deployment/fog-node -n openfaas',
            roles=roles["master"])
        log_cmd(env, res)

    res = en.run_command(
        'kubectl logs deployment/market -n openfaas',
        roles=roles["market"])
    log_cmd(env, res)


@cli.command()
@click.option('--file', required=False, help="Write output to file")
@enostask()
def openfaas_login(env=None, file=None, **kwargs):
    """Get OpenFaaS login info.

    Username is always `admin`.
    """
    roles = env['roles']
    res = en.run_command(
        'echo -n $(kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo)',
        roles=roles["master"])
    log_cmd(env, res)
    if file:
        with open(file, 'w') as f:
            f.write(str(res.filter(status=STATUS_OK).data[0].payload['stdout']) + "\n")


@cli.command()
@click.option('--all', required=False, is_flag=True, help="Also tunnel fog nodes")
@enostask()
def tunnels(env=None, all=False, **kwargs):
    """Open the tunnels to the K8S UI and to OpenFaaS from the current host."""
    roles = env['roles']
    if all:
        for role in roles['master']:
            address = role.address
            open_tunnel(address, 31112)  # OpenFaas
            open_tunnel(address, 3030)  # Fog Node
            open_tunnel(address, 8001,
                        "/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/#/node?namespace=default")  # K8S API

    for role in roles['market']:
        address = role.address

        open_tunnel(address, 8000)  # Market

    open_tunnel(env['roles']['prom_master'][0].address, 9090)
    open_tunnel(env['monitor'].ui.address, 3000)
    open_tunnel(env['roles']['iot_emulation'][0].address, 7070)

    print("Press Enter to kill.")
    input()


@cli.command()
@enostask()
def endpoints(env=None, **kwargs):
    """List the address of the end-nodes in the Fog network"""
    roles = env["roles"]
    for extremity in EXTREMITIES:
        role = roles[extremity][0]
        address = role.address
        print(f'{extremity} -> {address}')

    print(f"---\nIot emulation IP -> {roles['iot_emulation'][0].address}")



@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env['provider']

    provider.destroy()


if __name__ == "__main__":
    cli()
