import logging
import os
from pathlib import Path
import subprocess

import click  # type: ignore
import enoslib as en  # type: ignore

# Enable rich logging
from enoslib import enostask  # type: ignore

log = logging.getLogger("rich")


@click.group()
def cli(**kwargs):
    """Experiment with k3s in G5K.

    Don't forget to clean with the `clean` verb.

    P.S.
    Errors with ssh may arise, consider `ln -s ~/.ssh/id_ed25519.pub ~/.ssh/id_rsa.pub` if necessary.
    """
    en.init_logging(level=logging.INFO)
    en.set_config(g5k_auto_jump=False, ansible_forks=10)


@cli.command()
@click.option("--force", is_flag=True, help="destroy and up")
@click.option("--name", help="The name of the job")
@click.option("--walltime", help="The wallime: hh:mm:ss")
@enostask(new=True)
def up(
    force,
    name="Nix❄️+En0SLib FTW ❤️",
    walltime="2:00:00",
    env=None,
    **kwargs,
):
    """Claim the resources and setup k3s."""
    env["MASTER_CLUSTER"] = os.environ["MASTER_CLUSTER"]
    cluster = env["MASTER_CLUSTER"]

    nb_cpu_per_machine = en.infra.enos_g5k.g5k_api_utils.get_threads(
        env["MASTER_CLUSTER"]
    )
    mem_per_machine = 1024 * 16

    print(f"Deploying on {cluster}, force: {force}")

    conf = en.VMonG5kConf.from_settings(
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

    provider = en.VMonG5k(conf)
    env["provider"] = provider

    roles, networks = provider.init(force_deploy=force)

    job = provider.g5k_provider.jobs[0]

    # get the ips to white list
    ips = [vm.address for vm in roles["master"]]

    city = en.infra.enos_g5k.g5k_api_utils.get_cluster_site(
        os.environ["MASTER_CLUSTER"]
    )

    # add ips to the white list for the job duration
    en.g5k_api_utils.enable_home_for_job(job, ips)

    username = en.g5k_api_utils.get_api_username()
    print(f"Mounting home of {username} on ips {ips}")

    roles = en.sync_info(roles, networks)

    env["roles"] = roles
    env["networks"] = networks

    subprocess.run(
        [
            "ssh",
            f"{username}@{city}.grid5000.fr",
            "rm ~/ca-certificates.crt; cp /etc/ssl/certs/ca-certificates.crt ~/ca-certificates.crt",
        ]
    )

    with en.actions(roles=roles["master"]) as a:
        a.shell("mkdir -p /nfs/{metrics-arks,logs,logs_campaign,experiment}")
        a.shell("touch /nfs/joblog")
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
        a.shell(f"touch /root/.python-grid5000.yaml")
        with open(Path.home() / ".python-grid5000.yaml") as file:
            for line in file:
                a.shell(f"echo '{line}' >> /root/.python-grid5000.yaml")
        a.shell(f"chmod 600 /root/.python-grid5000.yaml")


@cli.command()
def get_city():
    city = en.infra.enos_g5k.g5k_api_utils.get_cluster_site(
        os.environ["MASTER_CLUSTER"]
    )
    print(city)


@cli.command()
def get_username():
    username = en.g5k_api_utils.get_api_username()
    print(username)


@cli.command()
@click.option("--variations", help="Docker variation tags", required=False)
@enostask()
def run_command(env=None, variations: str | None = None, **kwargs):
    roles = env["roles"]
    en.run_command(
        "cd /home/enos; . /home/enos/env.source; tmux new -d bash -c 'cd /home/enos; . /home/enos/env.source; eval $(ssh-agent -s); ssh-add; just master_docker_campaign 2> ./logs_campaign/out.logs'",
        task_name="Run command experiment",
        roles=roles["master"],
    )


@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env["provider"]

    provider.destroy()


if __name__ == "__main__":
    cli()
