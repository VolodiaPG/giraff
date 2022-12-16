import base64
from datetime import datetime
import logging
import os
import signal
import subprocess
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

from monitoring import monitoring as mon
from k3s import K3s
from definitions import *

log = logging.getLogger("rich")

KUBECONFIG_LOCATION_K3S = "/etc/rancher/k3s/k3s.yaml"

TELEGRAF_IMAGE = "ghcr.io/volodiapg/telegraf:latest"
PROMETHEUS_IMAGE = "ghcr.io/volodiapg/prometheus:latest"
GRAFANA_IMAGE = "ghcr.io/volodiapg/grafana:latest"


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
        alias = role[0].address + ":3003"
        ret[alias] = node
    ret[roles["market"][0].address + ":3003"] = "market"

    return ret


def log_cmd(env, results_list):
    # if results.filter(status=STATUS_FAILED):
    #     for data in results.filter(status=STATUS_FAILED).data:
    #         data = data.payload
    #         if data["stdout"]:
    #             log.error(data["stdout"])
    #         if data["stderr"]:
    #             log.error(data["stderr"])

    # if results.filter(status=STATUS_OK):
    now = datetime.now()
    current_time = now.strftime("%Y-%m-%d-%H-%M-%S")
    prefix_dir = f"{os.getcwd()}/logs"
    prefix_simlink = f"{os.getcwd()}"
    try:
        os.mkdir(prefix_dir)
    except FileExistsError:
        pass
    path = f"{prefix_dir}/{current_time}"
    os.mkdir(path)
    try:
        os.remove(f"{prefix_simlink}/logs-latest")
    except (FileExistsError, FileNotFoundError):
        pass
    os.symlink(path, f"{prefix_simlink}/logs-latest")
    aliases = {}
    for results in results_list:
        for data in results.filter(status=STATUS_OK) + results.filter(
            status=STATUS_FAILED
        ):
            host = data.host
            data = data.payload
            alias_name = get_aliases(env).get(host, host)
            aliases[alias_name] = aliases.get(alias_name, -1) + 1
            alias_name = alias_name + (
                "" if aliases[alias_name] == 0 else "." + str(aliases[alias_name])
            )

            if data["stdout"]:
                # print(data["stdout"])
                with open(path + "/" + alias_name + ".log", "w") as file:
                    file.write(data["stdout"])

            if data["stderr"]:
                with open(path + "/" + alias_name + ".err", "w") as file:
                    file.write(data["stderr"])
                log.error(data["stderr"])

            try:
                subprocess.run(
                    [
                        "mprocs",
                        "--server",
                        "127.0.0.1:4050",
                        "--ctl",
                        f'{{c: add-proc, cmd: "echo {alias_name} && cat {path + "/" + alias_name + ".log"}}}',
                    ]
                )
            except:
                log.warning("Cannot use mprocs to output nice things organized.")


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
    en.init_logging(level=logging.INFO)


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

    print("Setting up network...")

    # netem = en.Netem()
    # env["netem"] = netem
    # establish_netem(env)

    print("Setting up k3s and FaaS...")

    # k3s = en.K3s(master=roles["master"], agent=list())
    k3s = K3s(master=roles["master"], agent=list())
    k3s.deploy()

    docker = en.Docker(agent=roles["iot_emulation"] + roles["prom_master"])
    docker.deploy()

    with actions(roles=roles["master"], gather_facts=False) as p:
        p.shell(
            """curl https://baltocdn.com/helm/signing.asc | gpg --dearmor | tee /usr/share/keyrings/helm.gpg > /dev/null
            apt-get install apt-transport-https --yes
            echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/helm.gpg] https://baltocdn.com/helm/stable/debian/ all main" | tee /etc/apt/sources.list.d/helm-stable-debian.list
            apt-get update
            apt-get install helm"""
        )
        p.shell(
            (
                f"""export KUBECONFIG={KUBECONFIG_LOCATION_K3S} \
                    && k3s kubectl apply -f https://raw.githubusercontent.com/openfaas/faas-netes/master/namespaces.yml \
                    && helm repo add openfaas https://openfaas.github.io/faas-netes/ \
                    &&  helm repo update \
                    && helm upgrade openfaas --install openfaas/openfaas \
                        --namespace openfaas  \
                        --set functionNamespace=openfaas-fn \
                        --set generateBasicAuth=true \
                        --set prometheus.image=ghcr.io/volodiapg/prometheus:latest \
                        --set alertmanager.image=ghcr.io/volodiapg/alertmanager:latest \
                        --set stan.image=ghcr.io/volodiapg/nats-streaming:latest \
                        --set nats.metrics.image=ghcr.io/prometheus-nats-exporter:latest"""
            ),
            task_name="[master] Installing OpenFaaS",
        )
        p.shell(
            f"k3s kubectl port-forward -n openfaas svc/gateway 8080:8080",
            background=True,
        )


@cli.command()
@enostask()
def iot_emulation(env=None, **kwargs):
    roles = env["roles"]
    # Deploy the echo node
    with actions(roles=roles["iot_emulation"], gather_facts=False) as p:
        p.shell(
            f"""(docker stop iot_emulation || true) \
                && (docker rm iot_emulation || true) \
                && docker pull ghcr.io/volodiapg/iot_emulation:latest \
                && docker run --name iot_emulation --env COLLECTOR_IP={roles["prom_master"][0].address} -p 3003:3003 ghcr.io/volodiapg/iot_emulation:latest""",
            task_name="Run iot_emulation on the endpoints",
            background=True,
        )


@cli.command()
@enostask()
def network(env=None):
    if "netem" in env:
        netem = env["netem"]
        netem.destroy()

    netem = en.Netem()
    netem = en.NetemHTB()
    env["netem"] = netem
    roles = env["roles"]

    # netem.add_constraints("delay 20ms", roles["fog_node"], symetric=True)
    # netem.add_constraints("delay 0ms", roles["iot_emulation"], symetric=True)
    gen_net(NETWORK, netem, roles)

    netem.deploy()
    netem.validate()


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
            # delay=str(0) + "ms",
            rate="1gbit",
            symmetric=True,
        )
        gen_net(child, netem, roles)


@cli.command()
@enostask()
def monitoring(env=None, **kwargs):
    """Remove the constraints on the network links"""
    roles = env["roles"]
    if "monitor" in roles:
        monitor = env["monitor"]
        monitor.destroy()

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

    with actions(roles=roles["prom_master"], gather_facts=False) as p:
        p.shell(
            """(docker stop jaeger || true)
            (docker rm jaeger || true)
            docker run -d --name jaeger \
                -e COLLECTOR_ZIPKIN_HTTP_PORT=9411 \
                -p 5775:5775/udp \
                -p 6831:6831/udp \
                -p 6832:6832/udp \
                -p 5778:5778 \
                -p 16686:16686 \
                -p 14268:14268 \
                -p 9411:9411 \
                jaegertracing/all-in-one:1.6"""
        )


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

    en.run_command(
        "k3s kubectl delete -f /tmp/node_conf.yaml || true",
        roles=roles["master"],
        task_name="Removing existing fog_node software",
    )

    en.run_command(
        "(k3s kubectl delete -f /tmp/market.yaml || true) && sleep 30",
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
            conf=base64.b64encode(bytes(conf, "utf-8")).decode("utf-8"),
            collector_ip=roles["prom_master"][0].address,
            node_name=name,
        )
        roles[name][0].set_extra(fog_node_deployment=deployment)

    roles[NETWORK["name"]][0].set_extra(
        market_deployment=MARKET_DEPLOYMENT.format(
            collector_ip=roles["prom_master"][0].address
        )
    )

    try:
        res = en.run_command(
            "cat << EOF > /tmp/node_conf.yaml\n"
            "{{ fog_node_deployment }}\n"
            "EOF\n"
            "k3s kubectl create -f /tmp/node_conf.yaml",
            roles=roles["master"],
            task_name="Deploying fog_node software",
        )
        log_cmd(env, [res])
    except EnosFailedHostsError as err:
        for host in err.hosts:
            payload = host.payload
            print(payload)

    try:
        res = en.run_command(
            "cat << EOF > /tmp/market.yaml\n"
            "{{ market_deployment }}\n"
            "EOF\n"
            "k3s kubectl create -f /tmp/market.yaml",
            roles=roles["market"],
            task_name="Deploying market software",
        )
        log_cmd(env, [res])
    except EnosFailedHostsError as err:
        for host in err.hosts:
            payload = host.payload
            if "stdout" in payload and payload["stdout"]:
                print(payload["sdout"])
            if "stderr" in payload and payload["stderr"]:
                log.error(payload["stderr"])

    # establish_netem(env)


@cli.command()
@click.option("--all", is_flag=True, help="all namespaces")
@enostask()
def health(env=None, all=False, **kwargs):
    roles = env["roles"]

    command = "kubectl get deployments -n openfaas"
    if all:
        command = "kubectl get deployments --all-namespaces"
    res = en.run_command(command, roles=roles["master"])
    log_cmd(env, [res])


@cli.command()
@enostask()
def functions(env=None, **kwargs):
    roles = env["roles"]
    res = en.run_command(
        "kubectl get deployments -n openfaas-fn", roles=roles["master"]
    )
    log_cmd(env, [res])


@cli.command()
@enostask()
def toto(env=None, **kwargs):
    roles = env["roles"]
    res = en.run_command(
        "k3s kubectl get pods -A",
        roles=roles["master"],
    )
    log_cmd(env, [res])


@cli.command()
@click.option("--all", is_flag=True, help="all namespaces")
@enostask()
def logs(env=None, all=False, **kwargs):
    roles = env["roles"]

    res = []

    res.append(
        en.run_command(
            "k3s kubectl logs deployment/market -n openfaas --container sidecar-logs",
            roles=roles["market"],
        )
    )
    res.append(
        en.run_command(
            "k3s kubectl logs deployment/market -n openfaas --container market",
            roles=roles["market"],
        )
    )
    res.append(
        en.run_command("docker logs iot_emulation", roles=roles["iot_emulation"])
    )
    if all:
        res.append(
            en.run_command(
                "k3s kubectl logs deployment/fog-node -n openfaas --container sidecar-logs",
                roles=roles["master"],
            )
        )
        res.append(
            en.run_command(
                "k3s kubectl logs deployment/fog-node -n openfaas --container fog-node",
                roles=roles["master"],
            )
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
    log_cmd(env, [res])
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
                print(f"Opening to {address}")
                open_tunnel(address, 31112)  # OpenFaas
                open_tunnel(address, 3003)  # Fog Node
                open_tunnel(
                    address,
                    8001,
                    "/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/#/node?namespace=default",
                )  # K8S API

        tun = {}  # out_port, (res of tunnels())
        flag = False
        for role in roles["market"]:
            res = open_tunnel(role.address, 3008)  # Market
            if flag == False:
                tun[8088] = res
                flag = True

        if "prom_master" in env["roles"]:
            tun[9090] = open_tunnel(env["roles"]["prom_master"][0].address, 9090)
            tun[9096] = open_tunnel(env["roles"]["prom_master"][0].address, 16686)
        if "monitor" in env:
            tun[7070] = open_tunnel(env["monitor"].ui.address, 3000)
        if "iot_emulation" in env["roles"]:
            tun[3003] = open_tunnel(env["roles"]["iot_emulation"][0].address, 3003)

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
        f"""curl -w @- -o /dev/null -X HEAD -s "http://{dest.address}:3003/api/health" <<'EOF'
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
    log_cmd(env, [res])


@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env["provider"]

    provider.destroy()


if __name__ == "__main__":
    cli()
