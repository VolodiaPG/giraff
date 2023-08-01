import base64
import csv
import heapq
from io import TextIOWrapper
import logging
import os
import signal
import subprocess
import tarfile
import tempfile
import time
import uuid
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from time import sleep
import multiprocessing as mp
import click
import enoslib as en
from influxdb_client import InfluxDBClient
from inspect import isfunction
from collect import listener, worker
from definitions import (
    ADJACENCY,
    EXTREMITIES,
    FOG_NODE_DEPLOYMENT,
    FOG_NODES,
    IOT_CONNECTION,
    LEVELS,
    MARKET_CONNECTED_NODE,
    MARKET_DEPLOYMENT,
    NB_CPU_PER_MACHINE_PER_CLUSTER,
    NETWORK,
    NODE_CONNECTED_NODE,
    adjacency_undirected,
    flatten,
)

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
        alias = role[0].address + ":30003"
        ret[alias] = node
    ret[roles["market"][0].address + ":30003"] = "market"
    ret[roles["market"][0].address + ":30008"] = "marketplace"

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
            except FileNotFoundError:
                log.warning("Cannot use mprocs to output nice things organized.")


def open_tunnel(address, port, local_port=None, rest_of_url=""):
    print(
        f"doing tunnels for {address}:{port} -> http://localhost:{local_port}{rest_of_url}"
    )
    if local_port is None:
        local_port = port
    for i in range(5):
        try:
            tunnel = en.G5kTunnel(address=address, port=port, local_port=local_port)
            local_address, local_port, _ = tunnel.start()
            print(
                f"tunnel opened: {port} -> http://localhost:{local_port}{rest_of_url}"
            )
            return local_address, local_port
        except Exception as e:
            if i == 4:
                raise e
            else:
                print(f"Encountered exception: {e}. Retrying in 30 seconds...")
                time.sleep(30)


@click.group()
def cli(**kwargs):
    """Experiment with k3s in G5K.

    Don't forget to clean with the `clean` verb.

    P.S.
    Errors with ssh may arise, consider `ln -s ~/.ssh/id_ed25519.pub ~/.ssh/id_rsa.pub` if necessary.
    """
    en.init_logging(level=logging.INFO)
    en.set_config(g5k_auto_jump=False, ansible_forks=32)


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
    # add the market
    vms[frozenset(NETWORK["flavor"].items())].append("market")
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
                    roles=["master", "prom_agent", vm_id],  # "fog_node"
                    cluster=cluster,
                    number=nb_vms,
                    flavour_desc={"mem": mem, "core": core},
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
                roles=["master", "prom_agent", vm_id],  # "fog_node",
                cluster=cluster,
                number=nb_vms,
                flavour_desc={"mem": mem, "core": core},
            )

    return attributions


def attributes_roles(vm_attributions, roles):
    count = defaultdict(lambda: 0)
    for vm, instance_id in vm_attributions.items():
        roles[vm] = [roles[instance_id][count[instance_id]]]
        count[instance_id] += 1


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


@cli.command()
@click.option("--force", is_flag=True, help="destroy and up")
@click.option("--name", help="The name of the job")
@click.option("--walltime", help="The wallime: hh:mm:ss")
@click.option("--dry-run", is_flag=True, help="Do not reserve")
@enostask(new=True)
def up(
    force,
    name="Nix❄️+En0SLib FTW ❤️",
    walltime="2:00:00",
    dry_run=False,
    env=None,
    **kwargs,
):
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

    print(f"Deploying on {cluster}, force: {force}")

    conf = (
        en.VMonG5kConf.from_settings(
            job_name=name,
            walltime=walltime,
            image="/home/volparolguarino/nixos.qcow2",
            reservation=os.environ["RESERVATION"]
            if "RESERVATION" in os.environ
            else None,
        )
        # .add_machine(
        #     roles=["prom_master"],
        #     cluster=cluster,
        #     number=1,
        #     flavour_desc={"core": nb_cpu_per_machine, "mem": mem_per_machine},
        # )
        .add_machine(
            roles=["prom_agent", "iot_emulation"],
            cluster=cluster,
            number=1,
            flavour_desc={"core": nb_cpu_per_machine, "mem": mem_per_machine},
        )
    )

    assignations = assign_vm_to_hosts(
        NETWORK, conf, cluster, nb_cpu_per_machine, mem_per_machine
    )

    env["assignations"] = assignations

    print(
        f"I need {len(conf.machines)} bare-metal nodes in total, running a total of {len(assignations)} Fog node VMs"
    )

    conf.finalize()

    if dry_run:
        return

    provider = en.VMonG5k(conf)
    env["provider"] = provider

    time.sleep(10)

    # Encapsulate the code block in a try-except block and a for loop to retry 5 times
    for i in range(5):
        try:
            roles, networks = provider.init(force_deploy=force)
            break
        except Exception as e:
            if i == 4:
                raise e
            else:
                print(f"Encountered exception: {e}. Retrying in 30 seconds...")
                time.sleep(30)

    # Encapsulate the code block in a try-except block and a for loop to retry 5 times
    for i in range(5):
        try:
            en.wait_for(roles)

            roles = en.sync_info(roles, networks)

            attributes_roles(assignations, roles)

            roles = en.sync_info(roles, networks)

            env["roles"] = roles
            env["networks"] = networks
            break
        except Exception as e:
            if i == 4:
                raise e
            else:
                print(f"Encountered exception: {e}. Retrying in 30 seconds...")
                time.sleep(30)


@cli.command()
@enostask()
def restart(env=None):
    """
    Restarts the VMs, because they are Stateless NixOS instances, rebooting will umount all tmpfs (aka /) and will reset everything, except some stuff
    
    Because of some random shit, it looks like once in a while an instance reboots and then refuses to join back the network, the problem seems to be inside the VM itself
    To mitigate, it is adviced to run the restart command with and or (||) with a deploy command to re-deploy in the case of failure.
    However, precautions have been taken in this function to only reboot one VM at a time per host (though only waiting a small amount of time before passing to the next)
    """
    roles = env["roles"]["master"]
    inv_map = {}
    layers = []
    for k, v in env["assignations"].items():
        inv_map[v] = inv_map.get(v, []) + [k]
    for k, v in inv_map.items():
        for ii, el in enumerate(env["roles"][k]):
            if ii >= len(layers):
                layers.append([el])
            else:
                layers[ii] = layers[ii] + [el]   

    for hosts in layers:
        with actions(roles=hosts, gather_facts=False, strategy="free", background=True) as p:
            p.wait_for(retries = 5)
            p.shell('touch /iwasthere', task_name="Create iwasthere checkfile")
            p.shell('nohup sh -c "sleep 1; shutdown 0 -r"', task_name="Rebooting")
            sleep(10)
    # with actions(roles=roles, gather_facts=False, strategy="free", background=True) as p:
    #     p.wait_for(retries = 5)
    #     p.shell('touch /iwasthere', task_name="Create iwasthere checkfile")
    #     p.shell('nohup sh -c "sleep 1; shutdown 0 -r"', task_name="Rebooting")
    #     sleep(20)
    sleep(20)
    en.wait_for(roles=roles, retries=15)

    with actions(roles=roles, gather_facts=False, strategy="free") as p:
            p.shell('bash -c "[ ! -f /iwasthere ]"', task_name="Checking if reboot took effect, aka is iwasthere is no more")


@cli.command()
@enostask()
def k3s_setup(env=None):
    roles = env["roles"]
    print("Setting up k3s and FaaS...")

    with actions(roles=roles["master"], gather_facts=False) as p:
        p.shell(
            (
                f"""export KUBECONFIG={KUBECONFIG_LOCATION_K3S} \
                    && until k3s kubectl wait pods -n openfaas -l app=gateway --for condition=Ready --timeout=10s; do sleep 10; done"""
            ),
            task_name="[master] Installing OpenFaaS",
        )


@cli.command()
@enostask()
def iot_emulation(env=None, **kwargs):
    roles = env["roles"]
    # Deploy the echo node
    with actions(roles=roles["iot_emulation"], gather_facts=False) as p:
        p.shell(
            # --env INFLUX_ADDRESS="{roles["prom_master"][0].address}:9086" \
            f"""(docker stop iot_emulation || true) \
                && (docker rm iot_emulation || true) \
                && docker pull ghcr.io/volodiapg/iot_emulation:latest \
                && docker run --name iot_emulation \
                    --env INFLUX_ADDRESS="10.42.0.1:9086" \
                    --env INFLUX_TOKEN="xowyTh1iGcNAZsZeydESOHKvENvcyPaWg8hUe3tO4vPOw_buZVwOdUrqG3gwV314aYd9SWKHcxlykcQY_rwYVQ==" \
                    --env INFLUX_ORG="faasfog" \
                    --env INFLUX_BUCKET="faasfog"  \
                    --env INSTANCE_NAME="iot_emulation" \
                    -p 3003:3003 ghcr.io/volodiapg/iot_emulation:latest""",
            task_name="Run iot_emulation on the endpoints",
            background=True,
        )


@cli.command()
@enostask()
def network(env=None):
    for i in range(15):
        try:
            if "netem" in env:
                netem = env["netem"]
                netem.destroy()

            # netem = en.AccurateNetemHTB()
            netem = en.NetemHTB()
            env["netem"] = netem
            roles = env["roles"]
            # netem = None
            # roles = None

            gen_net(NETWORK, netem, roles)

            netem.deploy()
            netem.validate()
            break
        except Exception as e:
            if i == 5:
                raise e
            else:
                print(f"Encountered exception: {e}. Retrying in 30 seconds...")
                time.sleep(30)


def gen_net(nodes, netem, roles):
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
        pq = []
        heapq.heappush(pq, (0, src))

        # Create a vector for distances and initialize all
        # distances as infinite (INF)
        dist = defaultdict(lambda: float("inf"))
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
            netem.add_constraints(
                src=roles[node_name],
                dest=roles[destination],
                delay=str(latency) + "ms",  # That's a really bad fix there...
                # delay="20ms",
                rate="1gbit",
                # rate="100mbit",
                symmetric=True,
            )


@cli.command()
@enostask()
def monitoring(env=None, **kwargs):
    """Remove the constraints on the network links"""
    roles = env["roles"]
    # if "monitor" in roles:
    #     monitor = env["monitor"]
    #     monitor.destroy()

    # monitor = mon.TPGMonitoring(
    #     collector=roles["prom_master"][0],
    #     agent=roles["prom_agent"],
    #     ui=roles["prom_master"][0],
    #     telegraf_image=TELEGRAF_IMAGE,
    #     prometheus_image=PROMETHEUS_IMAGE,
    #     grafana_image=GRAFANA_IMAGE,
    # )
    # monitor.deploy()
    # env["monitor"] = monitor

    # with actions(roles=roles["prom_master"], gather_facts=False) as p:
    #     p.shell(
    #         """(docker stop jaeger || true)
    #         (docker rm jaeger || true)
    #         docker run -d --name jaeger \
    #             -e COLLECTOR_ZIPKIN_HTTP_PORT=9411 \
    #             -p 5775:5775/udp \
    #             -p 6831:6831/udp \
    #             -p 6832:6832/udp \
    #             -p 5778:5778 \
    #             -p 16686:16686 \
    #             -p 14268:14268 \
    #             -p 9411:9411 \
    #             quay.io/jaegertracing/all-in-one:1.41"""
    #     )
    pass


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
        reserved_cpu=node["flavor"]["reserved_core"],
        reserved_memory=node["flavor"]["reserved_mem"],
    )

    children = node["children"] if "children" in node else []

    return [
        (node["name"], conf, node["flavor"]),
        *[gen_conf(node, my_id, my_ip, ids) for node in children],
    ]


@cli.command()
@click.option(
    "--fog_node_image",
    help="The container image URL. eg. ghcr.io/volodiapg/fog_node:latest",
)
@click.option(
    "--market_image",
    help="The container image URL. eg. ghcr.io/volodiapg/market:latest",
)
@enostask()
def k3s_deploy(fog_node_image, market_image, env=None, **kwargs):
    roles = env["roles"]

    # en.run_command(
    #     "k3s kubectl delete -f /tmp/node_conf.yaml || true",
    #     roles=roles["master"],
    #     task_name="Removing existing fog_node software",
    # )

    # en.run_command(
    #     "(k3s kubectl delete -f /tmp/market.yaml || true) && sleep 30",
    #     roles=roles["master"],
    #     task_name="Removing existing market software",
    # )

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
                reserved_memory=NETWORK["flavor"]["reserved_mem"],
                reserved_cpu=NETWORK["flavor"]["reserved_core"],
            ),
            NETWORK["flavor"],
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

    for name, conf, tier_flavor in confs:
        pricing_cpu_initial = tier_flavor["pricing_cpu_initial"]
        pricing_cpu_initial = pricing_cpu_initial() if callable(pricing_cpu_initial) else pricing_cpu_initial
        pricing_mem_initial = tier_flavor["pricing_mem_initial"]
        pricing_mem_initial = pricing_mem_initial() if callable(pricing_mem_initial) else pricing_mem_initial
        
        deployment = FOG_NODE_DEPLOYMENT.format(
            conf=base64.b64encode(bytes(conf, "utf-8")).decode("utf-8"),
            # influx_ip=roles["prom_master"][0].address,
            influx_ip="10.42.0.1",
            node_name=name,
            fog_node_image=fog_node_image,
            pricing_cpu=tier_flavor["pricing_cpu"],
            pricing_mem=tier_flavor["pricing_mem"],
            pricing_cpu_initial=pricing_cpu_initial,
            pricing_mem_initial=pricing_mem_initial,
            pricing_geolocation=tier_flavor["pricing_geolocation"],
            is_cloud="is_cloud"
            if tier_flavor.get("is_cloud") is not None
            and tier_flavor.get("is_cloud") is True
            else "no_cloud",
        )
        # print(f"Doing name {name}")
        roles[name][0].set_extra(fog_node_deployment=deployment)

    roles[NETWORK["name"]][0].set_extra(
        market_deployment=MARKET_DEPLOYMENT.format(
            # influx_ip=roles["prom_master"][0].address,
            influx_ip="10.42.0.1",
            market_image=market_image,
        )
    )

    en.run_command(
        "cat << EOF > /tmp/node_conf.yaml\n"
        "{{ fog_node_deployment }}\n"
        "EOF\n"
        "k3s kubectl create -f /tmp/node_conf.yaml",
        roles=roles["master"],
        task_name="Deploying fog_node software",
    )
    en.run_command(
        "cat << EOF > /tmp/market.yaml\n"
        "{{ market_deployment }}\n"
        "EOF\n"
        "k3s kubectl create -f /tmp/market.yaml",
        roles=roles["market"],
        task_name="Deploying market software",
    )


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

def names(queue):
    names = aliases()
    with tempfile.NamedTemporaryFile(delete=False) as tmpfile:
        with TextIOWrapper(tmpfile, encoding="utf-8") as file:
            writer = csv.writer(file, delimiter="\t")
            writer.writerow(["instance", "name"])
            for key, value in names.items():
                writer.writerow([key, value])
        tmpfile.close()  # Close before sending to threads
        queue.put(("names", tmpfile.name))

def number_of_functions(queue):
    with tempfile.NamedTemporaryFile(delete=False) as tmpfile:
        with TextIOWrapper(tmpfile, encoding="utf-8") as file:
            low_latency = int(os.getenv("NB_FUNCTIONS_LOW_LATENCY"))
            rest_latency = int(os.getenv("NB_FUNCTIONS_REST"))
            
            high_load_low_latency =int(os.getenv("NB_FUNCTIONS_LOW_REQ_INTERVAL_LOW_LATENCY"))
            high_load_high_latency = low_latency - high_load_low_latency
            low_load_low_latency = int(os.getenv("NB_FUNCTIONS_HIGH_REQ_INTERVAL_LOW_LATENCY"))
            low_load_high_latency = rest_latency - low_load_low_latency

            writer = csv.writer(file, delimiter="\t")
            writer.writerow(["instance", "load", "latency", "value"])

            for source, _lat in IOT_CONNECTION:
                writer.writerow([source, "high", "low", high_load_low_latency])
                writer.writerow([source, "high", "high", high_load_high_latency])
                writer.writerow([source, "low", "low", low_load_low_latency])
                writer.writerow([source, "low", "high", low_load_high_latency])
        tmpfile.close()  # Close before sending to threads
        queue.put(("nb_functions", tmpfile.name))

def network_shape(queue):
    with tempfile.NamedTemporaryFile(delete=False) as tmpfile:
        with TextIOWrapper(tmpfile, encoding="utf-8") as file:
            writer = csv.writer(file, delimiter="\t")
            writer.writerow(["source", "destination", "latency"])
            for source, tup in ADJACENCY.items():
                for destination, latency in tup:
                    writer.writerow([source, destination, latency])
        tmpfile.close()  # Close before sending to threads
        queue.put(("network_shape", tmpfile.name))

def network_node_levels(queue):
    with tempfile.NamedTemporaryFile(delete=False) as tmpfile:
        with TextIOWrapper(tmpfile, encoding="utf-8") as file:
            writer = csv.writer(file, delimiter="\t")
            writer.writerow(["source", "level"])
            for source, level in LEVELS.items():
                writer.writerow([source, level])
        tmpfile.close()  # Close before sending to threads
        queue.put(("node_levels", tmpfile.name))


@enostask()
def _collect(env=None, **kwargs):
    return env["agent_tunnels"]


@cli.command()
@click.option("--address", help="A particular address to look at")
def collect(address=None, **kwargs):
    if address is None:
        addresses = set(_collect(**kwargs))
    else:
        addresses = set([address])
    token = os.getenv("INFLUX_TOKEN")
    org = os.getenv("INFLUX_ORG")
    bucket = os.getenv("INFLUX_BUCKET")

    today = datetime.today()
    today = today.strftime("%Y-%m-%d-%H-%M")
    prefix_dir = "metrics-arks"
    prefix_filename = os.getenv("COLLECT_ARCHIVE_NAME")
    if prefix_filename is None:
        prefix_filename = ""
    try:
        os.mkdir(prefix_dir)
    except FileExistsError:
        pass
    archive = f"{prefix_dir}/metrics_{prefix_filename}_{today}.tar.xz"
    measurements = set()
    for address in addresses:
        with InfluxDBClient(url="http://" + address, token=token, org=org) as client:
            query = f"""import "influxdata/influxdb/schema"
                schema.measurements(bucket: "{bucket}")
            """
            tables = client.query_api().query(query)
            measurements.update([list(value)[2] for value in tables.to_values()])
    print(measurements)
    manager = mp.Manager()
    queue = manager.Queue()
    pool = mp.Pool(mp.cpu_count() + 2)

    # put listener to work first
    pool.apply_async(listener, (queue, archive))
    pool.apply_async(names, (queue,))
    pool.apply_async(network_shape, (queue,))
    pool.apply_async(network_node_levels, (queue,))
    pool.apply_async(number_of_functions, (queue,))

    jobs = []
    for measurement_name in measurements:
        job = pool.apply_async(
            worker, (queue, addresses, token, bucket, org, measurement_name)
        )
        jobs.append(job)

    # collect results from the workers through the pool result queue
    for job in jobs:
        if job is not None:
            job.get()

    queue.put("kill")
    pool.close()
    pool.join()

    print(f"Finished writing archive {archive}")

    try:
        os.remove("latest_metrics.tar.xz")
    except (FileExistsError, FileNotFoundError):
        pass
    os.symlink(archive, "latest_metrics.tar.xz")


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


@enostask()
def do_open_tunnels(env=None, **kwargs):
    roles = env["roles"]
    # open_tunnel(roles["market"][0].address, 30008, 8088)  # Market
    # if "prom_master" in env["roles"]:
    #     open_tunnel(env["roles"]["prom_master"][0].address, 9086, 9086)
    # if "iot_emulation" in roles:
    #     open_tunnel(roles["iot_emulation"][0].address, 3003)

    env["agent_tunnels"] = list()
    for prom_agent in roles["prom_agent"]:
        # local_address, local_port = open_tunnel(prom_agent.address, 9086, 0)
        # env["agent_tunnels"].append(f"{local_address}:{local_port}")
        env["agent_tunnels"].append(f"{prom_agent.address}:9086")


@cli.command()
@click.option(
    "--command",
    required=False,
    help="Pass command to execute once done, will close tunnels after task is exited",
)
def tunnels(command=None, **kwargs):
    """Open the tunnels to the K8S UI and to OpenFaaS from the current host."""
    # procs = []
    # try:
    do_open_tunnels()

    if command is not None:
        pro = subprocess.Popen(
            command,
            shell=True,
            preexec_fn=os.setsid,
        )
        pro.wait()
    else:
        sleep(1)
        print("Press Enter to kill.")
        input()
    # finally:
    #     for pro in procs:
    #         os.killpg(
    #             os.getpgid(pro.pid), signal.SIGTERM
    #         )  # Send the signal to all the process groups


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
@enostask()
def market_ip(env=None, **kwargs):
    """List the address of the end-nodes in the Fog network"""
    role = env["roles"]["market"][0]
    address = role.address
    print(f"address: {address}")

@cli.command()
def iot_connections(env=None, **kwargs):
    """List the endpoints name where IoT Emulation is connected to"""
    for name, _ in IOT_CONNECTION:
        print(f"{name}")


@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env["provider"]

    provider.destroy()


if __name__ == "__main__":
    cli()
