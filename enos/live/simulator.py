"""
Gas Station Refueling example

Covers:

- Resources: Resource
- Resources: Container
- Waiting for other processes

Scenario:
  A gas station has a limited number of gas pumps that share a common
  fuel reservoir. Cars randomly arrive at the gas station, request one
  of the fuel pumps and start refueling from that reservoir.

  A gas station control process observes the gas station's fuel level
  and calls a tank truck for refueling if the station's level drops
  below a threshold.

"""
from __future__ import annotations
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
import random
from typing import List, Tuple, Dict
from alive_progress import alive_bar
import simpy

MS = 1
SECS = 1000

RANDOM_SEED = 42
T_INTER = [1, 1000]  # Interval between car arrivals [min, max] (ms)
SIM_TIME = 1000 * SECS  # Simulation time (milliseconds)


class Request(ABC):
    @abstractmethod
    def __call__(self, node: FogNode, caller: str):
        pass


@dataclass
class SLA:
    mem: float
    core: float
    latency: float
    duration: float


@dataclass(order=True)
class Bid:
    sort_index: float = field(init=False)
    bid: float
    bidder: str

    def __post_init__(self):
        self.sort_index = self.bid


class AuctionBidRequest(Request):
    def __init__(
        self, env, sla: SLA, network: Dict[Tuple[str, str], float], acc_latency=0.0
    ) -> None:
        super().__init__()
        self.env = env
        self.acc_latency = acc_latency
        self.sla = sla
        self.network = network

    def __call__(self, node: FogNode, caller: str):
        bids = list()
        for child in node.children + [node.parent] if node.parent is not None else []:
            if child.name == caller:
                continue
            delay = self.network[(node.name, child.name)]
            if delay + self.acc_latency <= self.sla.latency:
                req = yield env.process(
                    node.send(
                        child,
                        AuctionBidRequest(
                            self.env, self.sla, self.network, self.acc_latency + delay
                        ),
                    )
                )
                bids.extend(req)

        if node.cores_used + self.sla.core <= node.cores:
            bids.append(Bid(1.0, node.name))

        return bids


class ProvisionRequest(Request):
    def __init__(self, env, monitoring: Monitoring, sla: SLA, price: float) -> None:
        super().__init__()
        self.env = env
        self.price = price
        self.sla = sla

    def __call__(self, node: FogNode, _caller: str) -> bool:
        # yield self.env.timeout(0)
        if node.cores_used + self.sla.core > node.cores:
            return False
        node.cores_used += self.sla.core
        node.provisioned.append(self.sla)
        monitoring.currently_provisioned += 1
        yield self.env.timeout(self.sla.duration)
        node.cores_used -= self.sla.core
        node.provisioned.remove(self.sla)
        monitoring.currently_provisioned -= 1
        return True


class FogNode(object):
    def __init__(
        self,
        env,
        network: Dict[Tuple[str, str], float],
        name: str,
        parent: FogNode | None,
        cores: float,
        mem: float,
    ):
        self.env = env
        self.network = network
        self.name = name
        self.children: list[FogNode] = []
        self.parent = parent

        self.cores = cores
        self.mem = mem
        self.cores_used = 0.0
        self.mem_used = 0.0
        self.provisioned: List[SLA] = []

    def add_children(self, node: FogNode):
        node.parent = self
        self.children.append(node)

    def send(self, destination: FogNode, request: Request):
        yield self.env.timeout(self.network[(self.name, destination.name)])
        ret = yield self.env.process(request(destination, self.name))
        return ret


class MarketPlace:
    def __init__(self, env, network: Dict[str, FogNode], monitoring: Monitoring):
        self.env = env
        self.network = network
        self.monitoring = monitoring

    def send(self, destination: str, request: Request):
        ret = yield env.process(request(self.network[destination], ""))
        return ret

    def auction(self, first_node: str, auction: AuctionBidRequest):
        bids = yield self.env.process(self.send(first_node, auction))

        if len(bids) == 0:
            return

        bids.sort()
        winner = bids[0]
        price = bids[1].bid if len(bids) > 1 else bids[0].bid

        ret = yield self.env.process(
            self.send(
                winner.bidder,
                ProvisionRequest(env, self.monitoring, auction.sla, price),
            )
        )
        return ret


@dataclass
class Monitoring:
    _currently_provisioned = 0
    total_functions = 0
    total_nb_functions_provisioned = 0

    @property
    def currently_provisioned(self):
        return self._currently_provisioned

    @currently_provisioned.setter
    def currently_provisioned(self, value):
        diff = value - self._currently_provisioned
        if diff > 0:
            self.total_nb_functions_provisioned += diff
        self._currently_provisioned = value


monitoring = Monitoring()


def submit_function(env, network, marketplace, sla: SLA, mon: Monitoring):
    yield env.process(marketplace.auction("edge", AuctionBidRequest(env, sla, network)))
    mon.total_functions += 1


def function_generator(env, marketplace, network, mon):
    """Generate new SLA describing functions that arrive at the marketplace"""
    while True:
        yield env.timeout(random.randint(*T_INTER))
        sla = SLA(200, 0.25, 1 * SECS, 800 * SECS)
        env.process(submit_function(env, network, marketplace, sla, mon))


def function_generator_ll(env, marketplace, network, mon):
    """Generate new SLA describing functions that arrive at the marketplace"""
    while True:
        yield env.timeout(random.randint(*T_INTER))
        sla = SLA(200, 0.25, 4 * MS, 800 * SECS)
        env.process(submit_function(env, network, marketplace, sla, mon))


# Setup and start the simulation
print("FaaS Fog")
random.seed(RANDOM_SEED)

env = simpy.Environment()

latencies = {
    ("cloud", "teclo"): 3 * MS,
    ("telco", "cloud"): 3 * MS,
    ("telco", "edge"): 3 * MS,
    ("edge", "telco"): 3 * MS,
    ("cloud", "edge"): 6 * MS,
    ("edge", "cloud"): 6 * MS,
}

cloud = FogNode(env, latencies, "cloud", None, 32, 1024 * 128)
telco = FogNode(env, latencies, "telco", cloud, 8, 1024 * 16)
edge = FogNode(env, latencies, "edge", telco, 2, 1024 * 4)

cloud.add_children(telco)
telco.add_children(edge)

network = {"cloud": cloud, "telco": telco, "edge": edge}

marketplace = MarketPlace(env, network, monitoring)

env.process(function_generator(env, marketplace, latencies, monitoring))
env.process(function_generator_ll(env, marketplace, latencies, monitoring))

# Execute!
# env.run(until=SIM_TIME)
with alive_bar(
    int(SIM_TIME / SECS), title="Simulating...", ctrl_c=False, dual_line=True
) as bar:
    for ii in range(SIM_TIME):
        if ii % SECS == 0:
            bar.text = f"--> Done {monitoring.currently_provisioned}, failed to provision {monitoring.total_functions} functions..."
            bar()
        env.run(until=1 + ii)


print(
    f"--> Done {monitoring.total_nb_functions_provisioned}, failed to provision {monitoring.total_functions} functions."
)
