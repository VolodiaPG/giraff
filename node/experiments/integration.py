import logging
from pathlib import Path

import click
import enoslib as en
# Enable rich logging
from enoslib import enostask
from enoslib.api import STATUS_FAILED, STATUS_OK, actions
from grid5000 import Grid5000
from grid5000.cli import auth

log = logging.getLogger("rich")

CREATE_PROXY = "kubectl proxy --address='0.0.0.0' --accept-hosts='.*'"
# use grep {CREATE_PROXY}
KEY = "kubectl proxy"
GUARD_PROXY = f"ps aux | grep '{KEY}' | grep -v '{KEY}'"


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
    """Claim the resources and setup k3s."""

    conf = (
        en
            .VMonG5kConf
            .from_settings(job_name="En0SLib FTW"
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
)
            .add_machine(
            roles=["master", "cloud"],
            cluster="paravance",
            number=1,
            # flavour="large"
        )
            .add_machine(
            roles=["master", "london", "fog_node"],
            cluster="paravance",
            number=1,
            # flavour="large"
        )
            .add_machine(
            roles=["master", "berlin", "fog_node"],
            cluster="paravance",
            number=1,
            # flavour="large"
        )
            .finalize()
    )

    provider = en.VMonG5k(conf)

    roles, networks = provider.init(force_deploy=force)

    env['provider'] = provider
    env['roles'] = roles
    env['networks'] = networks

    en.wait_for(roles)

    k3s = en.K3s(master=roles['master'], agent=list())

    env['k3s-token'] = k3s.deploy()

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
            ("export KUBECONFIG=/etc/rancher/k3s/k3s.yaml && sudo -E arkade install openfaas"),
            task_name="[master] Installing OpenFaaS"
        )
        p.shell(f"k3s kubectl port-forward -n openfaas svc/gateway 8080:8080", background=True)


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
def k3s_token(env=None, **kwargs):
    """Show the provisioned token to connect to K3S cluster"""
    print(f'k3S bearer token: "{env["k3s-token"]}"')


@cli.command()
@enostask()
def health(env=None, **kwargs):
    roles = env['roles']
    res = en.run_command(
        'kubectl get pods -n openfaas',
        roles=roles["master"])
    log_cmd(res)

    res = en.run_command(
        'kubectl get services -n openfaas',
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

        def open_tunnel(port):
            tunnel = en.G5kTunnel(address=address, port=port)
            local_address, local_port, _ = tunnel.start()

            file_name = f"./port_maps_{address}_{port}"
            print(
                f"[Mapped {port}:{local_port}]The service is running at http://localhost:{local_port} (wrote to file: {file_name})")

            with open(file_name, "w") as f:
                f.write(f"{local_address}:{local_port}\n")

        open_tunnel(31112)  # OpenFaas
        open_tunnel(8001)  # K8S API

    print("Press Enter to kill.")
    input()
    # print("Create a tunnel from your local machine to the head node:")
    # print(f"ssh -NL 8001:{roles['master'][0].address}:8001 access.grid5000.fr")
    # print(f"ssh -NL 8080:{roles['master'][0].address}:31112 access.grid5000.fr")


@cli.command()
@enostask()
def clean(env=None, **kwargs):
    """Destroy the provided environment"""
    provider = env['provider']

    provider.destroy()


if __name__ == "__main__":
    cli()
