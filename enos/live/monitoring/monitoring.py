import os
from pathlib import Path
from typing import List, Optional

from enoslib.api import run_ansible
from enoslib.objects import Host, Network, Roles
from enoslib.service.service import Service
from enoslib.service.utils import _set_dir

DEFAULT_UI_ENV = {"GF_SERVER_HTTP_PORT": "3000"}

DEFAULT_COLLECTOR_ENV = {"INFLUXDB_HTTP_BIND_ADDRESS": ":8086"}

DEFAULT_AGENT_IMAGE = "telegraf"

SERVICE_PATH = os.path.abspath(os.path.dirname(os.path.realpath(__file__)))

LOCAL_OUTPUT_DIR_TIG = Path("__enoslib_tig__")
LOCAL_OUTPUT_DIR_TPG = Path("__enoslib_tpg__")


def _get_address(host: Host, networks: Optional[List[Network]]) -> str:
    """Auxiliary function to get the IP address for the Host

    Args:
        host: Host information
        networks: List of networks
    Returns:
        str: IP address from host
    """
    if networks is None:
        return host.address

    address = host.filter_addresses(networks, include_unknown=False)

    if not address or not address[0].ip:
        raise ValueError(f"IP address not found. Host: {host}, Networks: {networks}")

    if len(address) > 1:
        raise ValueError(
            f"Cannot determine single IP address."
            f"Options: {address} Host: {host}, Networks: {networks}"
        )
    return str(address[0].ip.ip)


class TPGMonitoring(Service):
    def __init__(
        self,
        collector: Host,
        agent: List[Host],
        *,
        ui: Host = None,
        networks: List[Network] = None,
        remote_working_dir: str = "/builds/monitoring",
        backup_dir: Optional[Path] = None,
        telegraf_image:  str = "telegraf",
        prometheus_image: str = "prom/prometheus",
        grafana_image: str = "grafana/grafana",
    ):
        """Deploy a TPG stack: Telegraf, Prometheus, Grafana.

        This assumes a debian/ubuntu base environment and aims at producing a
        quick way to deploy a monitoring stack on your nodes. Except for
        telegraf agents which will use a binary file for armv7 (FIT/IoT-LAB).

        It's opinionated out of the box but allow for some convenient
        customizations.

        Args:
            collector: :py:class:`enoslib.Host` where the collector
                    will be installed
            ui: :py:class:`enoslib.Host` where the UI will be installed
            agent: list of :py:class:`enoslib.Host` where the agent
                    will be installed
            networks: list of networks to use for the monitoring traffic.
                        Agents will send their metrics to the collector using
                        this IP address. In the same way, the ui will use this IP to
                        connect to collector.
                        The IP address is taken from :py:class:`enoslib.Host`, depending
                        on this parameter:
                        - None: IP address = host.address
                        - List[Network]: Get the first IP address available in
                        host.extra_addresses which belongs to one of these networks
                        Note that this parameter depends on calling sync_network_info to
                        fill the extra_addresses structure.
                        Raises an exception if no or more than IP address is found
            remote_working_dir: path to a remote location that
                will be used as working directory
            backup_dir: path to a local directory where the backup will be stored
                This can be overwritten by
                :py:meth:`~enoslib.service.monitoring.monitoring.TPGMonitoring.backup`.
        """

        # Some initialisation and make mypy happy
        self.collector = collector
        assert self.collector is not None
        self.agent = agent
        assert self.agent is not None
        self.telegraf_image = telegraf_image
        assert self.telegraf_image is not None
        self.prometheus_image = prometheus_image
        assert self.prometheus_image is not None
        self.grafana_image = grafana_image
        assert self.grafana_image is not None

        self.ui = ui

        self.networks = networks
        self._roles = Roles()
        ui_list = [self.ui] if self.ui else []
        self._roles.update(
            prometheus=[self.collector], telegraf=self.agent, grafana=ui_list
        )
        self.remote_working_dir = remote_working_dir
        self.prometheus_port = 9090

        # backup_dir management
        self.backup_dir = _set_dir(backup_dir, LOCAL_OUTPUT_DIR_TPG)

        # We force python3
        # self.extra_vars = {"ansible_python_interpreter": "/usr/bin/python3"}
        self.extra_vars = {}

    def deploy(self):
        """Deploy the monitoring stack"""
        ui_address = ""
        if self.ui:
            ui_address = _get_address(self.ui, self.networks)

        extra_vars = {
            "enos_action": "deploy",
            "collector_type": "prometheus",
            "remote_working_dir": self.remote_working_dir,
            "collector_address": _get_address(self.collector, self.networks),
            "collector_port": self.prometheus_port,
            "ui_address": ui_address,
            "telegraf_targets": [_get_address(h, self.networks) for h in self.agent],
            "agent_image": self.telegraf_image,
            "grafana_image": self.grafana_image,
            "prometheus_image": self.prometheus_image,
        }
        extra_vars.update(self.extra_vars)
        _playbook = os.path.join(SERVICE_PATH, "monitoring.yml")
        run_ansible([_playbook], roles=self._roles, extra_vars=extra_vars)

    def destroy(self):
        """Destroy the monitoring stack.

        This destroys all the container and associated volumes.
        """
        extra_vars = {
            "enos_action": "destroy",
            "remote_working_dir": self.remote_working_dir,
        }
        extra_vars.update(self.extra_vars)
        _playbook = os.path.join(SERVICE_PATH, "monitoring.yml")

        run_ansible([_playbook], roles=self._roles, extra_vars=extra_vars)

    def backup(self, backup_dir: Optional[str] = None):
        """Backup the monitoring stack.

        Args:
            backup_dir (str): path of the backup directory to use.
                Will be used instead of the one set in the constructor.
        """
        _backup_dir = _set_dir(backup_dir, self.backup_dir)
        extra_vars = {
            "enos_action": "backup",
            "remote_working_dir": self.remote_working_dir,
            "collector_address": _get_address(self.collector, self.networks),
            "collector_port": self.prometheus_port,
            "backup_dir": str(_backup_dir),
        }
        extra_vars.update(self.extra_vars)
        _playbook = os.path.join(SERVICE_PATH, "monitoring.yml")

        run_ansible(
            [_playbook],
            roles=Roles(prometheus=self._roles["prometheus"]),
            extra_vars=extra_vars,
        )

    def __exit__(self, *args):
        # special case here, backup will suspend the execution of the database
        # and backup can occur.  we destroy afterwards
        self.backup()
        self.destroy()
