import logging
import os
import subprocess
from pathlib import Path
from typing import Any, Optional

import click  # type: ignore
from enoslib import set_config  # type: ignore
from enoslib import sync_info  # type: ignore
from enoslib import (  # type: ignore
    VMonG5k,
    VMonG5kConf,
    actions,
    enostask,
    g5k_api_utils,
    init_logging,
)
from enoslib.infra.enos_g5k.g5k_api_utils import (  # type: ignore
    get_cluster_site,
    get_threads,
)

EnosEnv = Optional[dict[str, Any]]

log = logging.getLogger("rich")
STRATEGY_FREE="free"

@click.group()
def cli(**kwargs):
    """Experiment with k3s in G5K.

    Don't forget to clean with the `clean` verb.

    P.S.
    Errors with ssh may arise, consider `ln -s ~/.ssh/id_ed25519.pub ~/.ssh/id_rsa.pub` if necessary.
    """
    #logging.basicConfig(level = logging.DEBUG)
    init_logging(level=logging.INFO)
    #set_config(g5k_auto_jump=False)
    set_config(ansible_stdout="noop")


@cli.command()# type: ignore
@click.option("--force", is_flag=True, help="destroy and up")
@click.option("--name", help="The name of the job")
@click.option("--walltime", help="The wallime: hh:mm:ss")
@enostask(new=True)
def up(
    force,
    name="Nix❄️+En0SLib FTW ❤️",
    walltime="2:00:00",
    env: EnosEnv = None,
    **kwargs,
):
    """Claim the resources and setup k3s."""
    if env is None:
        print("env is None")
        exit(1)

    env["MASTER_CLUSTER"] = os.environ["MASTER_CLUSTER"]
    cluster = env["MASTER_CLUSTER"]

    nb_cpu_per_machine = get_threads(env["MASTER_CLUSTER"])
    mem_per_machine = 1024 * 16

    print(f"Deploying on {cluster}, force: {force}")

    conf = VMonG5kConf.from_settings(
        job_name=name,
        walltime=walltime,
        image="/home/volparolguarino/nixos.env.qcow2",
        reservation=os.environ["MASTER_RESERVATION"]
        if "MASTER_RESERVATION" in os.environ
        else None,
    ).add_machine(
        roles=["master"],
        cluster=cluster,
        number=1,
        flavour_desc={"core": nb_cpu_per_machine, "mem": mem_per_machine},
    )

    conf.finalize()

    provider = VMonG5k(conf)
    env["provider"] = provider

    roles, networks = provider.init(force_deploy=force)

    job = provider.g5k_provider.jobs[0]  # type: ignore

    # get the ips to white list
    ips = [vm.address for vm in roles["master"]]

    city = get_cluster_site(os.environ["MASTER_CLUSTER"])

    # add ips to the white list for the job duration
    g5k_api_utils.enable_home_for_job(job, ips)

    username = g5k_api_utils.get_api_username()
    print(f"Mounting home of {username} on ips {ips}")
    
    roles = sync_info(roles, networks)

    env["roles"] = roles
    env["networks"] = networks

    subprocess.run(
        [
            "ssh",
            f"{username}@{city}.grid5000.fr",
            "rm ~/ca-certificates.crt; cp /etc/ssl/certs/ca-certificates.crt ~/ca-certificates.crt",
        ]
    )
    


    with actions(roles=roles["master"], strategy=STRATEGY_FREE) as a:
        a.shell("mkdir -p /nfs/{metrics-arks,logs,logs_campaign,experiment}")
        a.shell("touch /nfs/joblog")
        a.shell("mkdir -p /home/enos")
        a.shell(
            "ln -s /nfs/{metrics-arks,logs,logs_campaign,joblog,experiment} /home/enos"
        )
        a.shell(
            "rm /etc/ssl/certs/ca-certificates.crt; cp /nfs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt"
        )
        a.shell(
            f"""cat <<EOF >> /root/.ssh/config
Host *.grid5000.fr
    User {username}
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
    ForwardAgent yes
EOF
            """
        )
        a.shell("touch /root/.python-grid5000.yaml")
        with open(Path.home() / ".python-grid5000.yaml") as file:
            for line in file:
                a.shell(f"echo '{line}' >> /root/.python-grid5000.yaml")
        a.shell("chmod 600 /root/.python-grid5000.yaml")


@cli.command()# type: ignore
def get_city():
    city = get_cluster_site(os.environ["MASTER_CLUSTER"])
    print(city)


@cli.command()# type: ignore
def get_username():
    username = g5k_api_utils.get_api_username()
    print(username)


@cli.command()# type: ignore
@click.option("--variations", help="Docker variation tags", required=False)
@enostask()
def run_command(env: EnosEnv = None, variations: str | None = None, **kwargs):
    if env is None:
        print("env is None")
        exit(1)

    roles = env["roles"]
    with actions(roles=roles["master"], strategy=STRATEGY_FREE) as a:
        a.shell(
            "until [ -f /home/enos/env.source ]; do sleep 3; done; "
            "cd /home/enos;"
            ". /home/enos/env.source;"
            "tmux new -d bash -c "
            "'"
            "cd /home/enos;"
            ". /home/enos/env.source;"
            "eval $(ssh-agent -s);"
            "ssh-add;"
            "just master_docker_campaign > ./logs_campaign/out.logs 2>&1"
            "'",
            task_name="Run command experiment",
        )

    
@cli.command()# type: ignore
@click.option("--variations", help="Docker variation tags", required=False)
@enostask()
def run_command_refresh(env: EnosEnv = None, variations: str | None = None, **kwargs):
    if env is None:
        print("env is None")
        exit(1)

    roles = env["roles"]
    with actions(roles=roles["master"], strategy=STRATEGY_FREE) as a:
        a.shell(
            "until [ -f /home/enos/env.source ]; do sleep 3; done; "
            "cd /home/enos;"
            ". /home/enos/env.source;"
            "tmux new -d bash -c "
            "'"
            "cd /home/enos;"
            ". /home/enos/env.source;"
            "eval $(ssh-agent -s);"
            "ssh-add;"
            "just master_docker_campaign > ./logs_campaign/out.logs 2>&1"
            "'",
            task_name="Run command experiment",
        )


@cli.command()# type: ignore
@enostask()
def clean(env: EnosEnv = None, **kwargs):
    """Destroy the provided environment"""
    if env is None:
        print("env is None")
        exit(1)

    provider = env["provider"]

    provider.destroy()


if __name__ == "__main__":
    cli()
