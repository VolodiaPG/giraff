import base64
import logging
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

log = logging.getLogger("rich")

KUBECONFIG_LOCATION_K3S = "/etc/rancher/k3s/k3s.yaml"

FOG_NODE_DEPLOYMENT = """apiVersion: apps/v1
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
          value: "3000"
        - name: CONFIG
          value: "{conf}"
        ports:
        - containerPort: 3000
        command: ["sh"]
        args: ["-c", "fog_node <(echo \$CONFIG | base64 -d | cat)"]
"""

MARKET_DEPLOYMENT = """apiVersion: apps/v1
kind: Deployment
metadata
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
"""

MARKET_CONNECTED_NODE = """MarketConnected (
    market_ip: "{market_ip}",
    market_port: 8000,
    my_id: "{my_id}"
)
"""

NODE_CONNECTED_NODE = """NodeConnected (
    parent_id: "{parent_id}",
    parent_node_ip: "{parent_ip}",
    parent_node_port: 3000,
    my_id: "{my_id}"
)
"""

NETWORK = {
    "name": "market",
    "children": [
        {
            "name": "london",
            "children": [
                {
                    "name": "berlin",
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


FOG_NODES = list(flatten([gen_fog_nodes_names(child) for child in NETWORK["children"]]))


def log_cmd(results):
    if results.filter(status=STATUS_FAILED):
        for data in results.filter(status=STATUS_FAILED).data:
            data = data.payload
            if data['stdout']:
                log.error(data['stdout'])
            if data['stderr']:
                log.error(data['stderr'])

    if results.filter(status=STATUS_OK):
        for data in results.filter(status=STATUS_OK).data:
            data = data.payload
            if data['stdout']:
                log.info(data['stdout'])
            if data['stderr']:
                log.error(data['stderr'])


def open_tunnel(address, port, rest_of_url=""):
    tunnel = en.G5kTunnel(address=address, port=port)
    local_address, local_port, _ = tunnel.start()
    print(f"tunnel opened: http://localhost:{local_port}{rest_of_url}")
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


@cli.command()
@click.option("--force", is_flag=True, help="destroy and up")
@enostask(new=True)
def up(force, env=None, **kwargs):
    """Claim the resources and setup k3s."""

    conf = (
        en
            .VMonG5kConf
            .from_settings(job_name="En0SLib FTW")
            .add_machine(
            roles=["master", "market"],
            cluster="paravance",
            number=1,
            flavour="large"
        )
    )

    for node_name in FOG_NODES:
        conf.add_machine(
            roles=["master", "fog_node", node_name],
            cluster="paravance",
            number=1,
            flavour="large")

    conf.finalize()

    provider = en.VMonG5k(conf)

    roles, networks = provider.init(force_deploy=force)

    env['provider'] = provider
    env['roles'] = roles
    env['networks'] = networks

    en.wait_for(roles)

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
            # noqa
            task_name="token",
        )
        p.shell(f"k3s kubectl port-forward -n openfaas svc/gateway 8080:8080", background=True)
    env["k3s-token"] = [res.stdout for res in p.results.filter(task="token")]


@cli.command()
@enostask()
def add_net_constraints(env=None, **kwargs):
    """Constraint the network links"""
    roles = env['roles']
    networks = env['networks']
    roles = en.sync_info(roles, networks)

    netem = en.NetemHTB()

    env['netem'] = netem

    (
        netem.add_constraints(
            src=roles["cloud"],
            dest=roles["london"],
            delay="50ms",
            rate="1gbit",
            symetric=True,
        )
            .add_constraints(
            src=roles["cloud"],
            dest=roles["berlin"],
            delay="100ms",
            rate="1mbit",
            symetric=True,
        )
    )

    netem.deploy()


@cli.command()
@enostask()
def rm_net_constraints(env=None, **kwargs):
    """Remove the constraints on the network links"""
    netem = env['netem']
    netem.destroy()


@cli.command()
@enostask()
def k3s_config(env=None, **kwargs):
    """SCP the remote kubeconfig files"""
    for out in env["k3s-token"]:
        print(out)


def gen_conf(node, parent_id, parent_ip, ids):
    (my_id, my_ip) = ids[node["name"]]
    conf = NODE_CONNECTED_NODE.format(parent_id=parent_id, parent_ip=parent_ip, my_id=my_ip)

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
    confs = [(NETWORK["name"], MARKET_CONNECTED_NODE.format(market_ip=market_ip, my_id=market_id))]
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
        log_cmd(res)
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
        log_cmd(res)
    except EnosFailedHostsError as err:
        for host in err.hosts:
            payload = host.payload
            if payload['stdout']:
                print(payload['stdout'])
            if payload['stderr']:
                log.error(payload['stderr'])


@cli.command()
@enostask()
def health(env=None, **kwargs):
    roles = env['roles']
    res = en.run_command(
        'kubectl get deployments --all-namespaces',
        roles=roles["master"])
    log_cmd(res)


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
    log_cmd(res)
    if file:
        with open(file, 'w') as f:
            f.write(str(res.filter(status=STATUS_OK).data[0].payload['stdout']) + "\n")


@cli.command()
@enostask()
def tunnels(env=None, **kwargs):
    """Open the tunnels to the K8S UI and to OpenFaaS from the current host."""
    roles = env['roles']
    for role in roles['master']:
        address = role.address

        open_tunnel(address, 31112)  # OpenFaas
        open_tunnel(address, 8001,
                    "/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/#/node?namespace=default")  # K8S API

    print("Press Enter to kill.")
    input()


@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env['provider']

    provider.destroy()


if __name__ == "__main__":
    cli()
