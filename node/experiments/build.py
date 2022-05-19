import logging
import subprocess
from pathlib import Path

import click
import enoslib as en
# Enable rich logging
from enoslib import enostask
from enoslib.errors import EnosFailedHostsError
from grid5000 import Grid5000
from grid5000.cli import auth

log = logging.getLogger("rich")

CREATE_PROXY = "kubectl proxy --address='0.0.0.0' --accept-hosts='.*'"
# use grep {CREATE_PROXY}
KEY = "kubectl proxy"
GUARD_PROXY = f"ps aux | grep '{KEY}' | grep -v '{KEY}'"

# Where to find  the `manager` folder
PATH = ".." + "/"

def log_cmd(results):
    for res in results.data:
        payload = res.payload
        if payload['stdout']:
            log.info(payload['stdout'])
        if payload['stderr']:
            log.error(payload['stderr'])
    # if results.filter(status=STATUS_FAILED):
    #     for data in results.filter(status=STATUS_FAILED).data:
    #         data = data.payload
    #         if data['stdout']:
    #             log.error(data['stdout'])
    #         if data['stderr']:
    #             log.error(data['stderr'])
    #
    # if results.filter(status=STATUS_OK):
    #     for data in results.filter(status=STATUS_OK).data:
    #         data = data.payload
    #         if data['stdout']:
    #             log.info(data['stdout'])
    #         if data['stderr']:
    #             log.error(data['stderr'])


@click.group()
# @click.option('--logging/--no-logging', default=True, help="Enable/Disable logging")
def cli(**kwargs):
    """Experiment with k3s in G5K.

    Don't forget to clean with the `clean` verb.

    P.S.
    Errors with ssh may arise, consider `ln -s ~/.ssh/id_ed25519.pub ~/.ssh/id_rsa.pub` if necessary.
    """
    en.init_logging()
    # if kwargs['logging']:
    # else:
    #     en.init_logging(level=logging.CRITICAL)
    #     log.setLevel(level=logging.CRITICAL)


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
    network = en.G5kNetworkConf(type="prod", roles=["my_network"], site="lyon")

    conf = (
        en.G5kConf.from_settings(job_type="exotic", job_name="EnosLib FTW")
            .add_network_conf(network)
            .add_machine(
            roles=["builder"], cluster="neowise", nodes=1, primary_network=network
        )
            .finalize()
    )

    provider = en.G5k(conf)

    roles, networks = provider.init(force_deploy=force)

    env['provider'] = provider
    env['roles'] = roles
    env['networks'] = networks

    en.wait_for(roles)

    # with actions(roles=roles['builder'], gather_facts=False) as p:
    #     p.shell(
    #         ("docker build ."),
    #         task_name="[builder] Docker build"
    #     )


@cli.command()
@enostask()
def upload(env=None, **kwargs):
    """Destroy the provided environment"""
    roles = env['roles']
    #
    # res = en.run_command(
    #     'sudo mkdir manager && chmod 777 -R manager',
    #     roles=roles["builder"])
    # log_cmd(res)

    res = en.run_command(
        '(rm -rf /tmp/manager || true) && mkdir -p /tmp/manager && chmod 777 -R /tmp/manager',
        roles=roles["builder"], )
    log_cmd(res)

    address = roles['builder'][0].address
    subprocess.run(["scp", PATH + "manager/Cargo.toml", f"{address}:/tmp/manager/"])
    subprocess.run(["scp", PATH + "manager/Cargo.lock", f"{address}:/tmp/manager/"])
    subprocess.run(["scp", "-r", PATH + "manager/src", f"{address}:/tmp/manager/"])
    subprocess.run(["scp", PATH + "manager/Dockerfile", f"{address}:/tmp/manager/"])
    subprocess.run(["scp", PATH + "manager/.dockerignore", f"{address}:/tmp/manager/"])


@cli.command()
@enostask()
def build(env=None, **kwargs):
    """Destroy the provided environment"""
    roles = env['roles']

    with en.Docker(agent=roles["builder"]):
        try:
            res = en.run_command(
                'cd /tmp/manager '
                ' && docker build -t fog_node:latest --target fog_node .'
                ' && docker build -t market:latest --target market .'
                ' && docker save fog_node:latest > gzip > /tmp/fog_node_latest.tar.gz'
                ' && docker save market:latest > gzip > /tmp/market_latest.tar.gz',
                roles=roles["builder"])
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
def download(env=None, **kwargs):
    """Destroy the provided environment"""
    roles = env['roles']

    address = roles['builder'][0].address
    subprocess.run(["scp", f"{address}:/tmp/fog_node_latest.tar.gz", "./"])
    subprocess.run(["scp", f"{address}:/tmp/market_latest.tar.gz", "./"])


@cli.command()
@click.option("--user", help="The Github user to upload the image to", required=True)
@enostask()
def ghcr(user=None, env=None, **kwargs):
    """Upload to ghcr.io"""
    def do_it(image_name):
        subprocess.run(["bash", "-c", f"docker load < ./{image_name}_latest.tar.gz"])
        subprocess.run(["docker", "tag", f"{image_name}:latest", f"ghcr.io/{user}/{image_name}:latest"])
        subprocess.run(["docker", "push", f"ghcr.io/{user}/{image_name}:latest"])

    do_it("market")
    do_it("fog_node")


@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env['provider']

    provider.destroy()


if __name__ == "__main__":
    cli()
