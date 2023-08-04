from __future__ import annotations
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
import os
import random
from typing import Any, Generator, List, Tuple, Dict
from alive_progress import alive_bar
import simpy
from definitions import NETWORK, gen_net
import expe

MS = 1
SECS = 1000

SIM_TIME = expe.FUNCTION_RESERVATION_FINISHES_AFTER * SECS

RANDOM_SEED = expe.RANDOM_SEED
if RANDOM_SEED is not None:
    random.seed(RANDOM_SEED)


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


@dataclass
class Monitoring:
    _currently_provisioned = 0
    total_provisioned = 0
    total_submitted = 0

    @property
    def currently_provisioned(self):
        return self._currently_provisioned

    @currently_provisioned.setter
    def currently_provisioned(self, value):
        diff = value - self._currently_provisioned
        if diff > 0:
            self.total_provisioned += diff
        self._currently_provisioned = value


class Request(ABC):
    """A visitor pattern driven request. Can act upon a given Fog node in any way desired way"""

    @abstractmethod
    def __call__(self, node: FogNode, caller: str) -> Any:
        pass


class Pricing(ABC):
    """A visitor pattern driven cost estimator, from a given SLA"""

    @abstractmethod
    def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
        pass


class ConstantPricing(Pricing):
    def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
        return Bid(1.0, node.name)


class AuctionBidRequest(Request):
    def __init__(
        self,
        env,
        sla: SLA,
        network: Dict[Tuple[str, str], float],
        acc_latency=0.0,
    ) -> None:
        super().__init__()
        self.env = env
        self.acc_latency = acc_latency
        self.sla = sla
        self.network = network

    def __call__(self, node: FogNode, caller: str):
        bids: Any = []
        requests: List[Any] = []
        for child in node.children + [node.parent] if node.parent is not None else []:
            if child.name == caller:
                continue
            delay = self.network[(node.name, child.name)]
            if delay + self.acc_latency <= self.sla.latency:
                req = env.process(
                    node.send(
                        child,
                        AuctionBidRequest(
                            self.env,
                            self.sla,
                            self.network,
                            self.acc_latency + delay,
                        ),
                    )
                )
                requests.append(req)

        for req in requests:
            req = yield req
            bids.extend(req)

        if (
            node.cores_used + self.sla.core <= node.cores
            and node.mem_used + self.sla.mem <= node.mem
        ):
            bids.append(node.pricing_strat(node, self.sla, self.acc_latency))

        return bids


class EdgeWardRequest(Request):
    def __init__(
        self,
        env,
        sla: SLA,
        network: Dict[Tuple[str, str], float],
        acc_latency=0.0,
    ) -> None:
        super().__init__()
        self.env = env
        self.acc_latency = acc_latency
        self.sla = sla
        self.network = network

    def __call__(self, node: FogNode, caller: str):
        bids = list()
        if node.parent is None:
            return []

        if (
            node.cores_used + self.sla.core <= node.cores
            and node.mem_used + self.sla.mem <= node.mem
        ):
            bids.append(node.pricing_strat(node, self.sla, self.acc_latency))

        if len(bids) > 0:
            return bids

        parent = node.parent
        delay = self.network[(node.name, parent.name)]
        req = yield env.process(
            node.send(
                parent,
                EdgeWardRequest(
                    self.env,
                    self.sla,
                    self.network,
                    self.acc_latency + delay,
                ),
            )
        )

        return req


@dataclass(order=True)
class SortableFogNode:
    sort_index: float = field(init=False)
    node: FogNode
    latency: float

    def __post_init__(self):
        self.sort_index = self.latency


class EdgeFirstRequest(Request):
    def __init__(
        self,
        env,
        sla: SLA,
        network: Dict[Tuple[str, str], float],
        acc_latency=0.0,
    ) -> None:
        super().__init__()
        self.env = env
        self.acc_latency = acc_latency
        self.sla = sla
        self.network = network

    def __call__(self, node: FogNode, caller: str):
        if (
            node.cores_used + self.sla.core <= node.cores
            and node.mem_used + self.sla.mem <= node.mem
        ):
            return [node.pricing_strat(node, self.sla, self.acc_latency)]

        nodes: Any = []
        for child in node.children + [node.parent] if node.parent is not None else []:
            if child.name == caller:
                continue
            delay = self.network[(node.name, child.name)]
            if delay + self.acc_latency <= self.sla.latency:
                continue
            nodes.append(SortableFogNode(child, self.acc_latency + delay))

        nodes.sort()

        for node, _ in nodes:
            req = yield env.process(
                node.send(
                    child,
                    AuctionBidRequest(
                        self.env,
                        self.sla,
                        self.network,
                        self.acc_latency + delay,
                    ),
                )
            )
            if len(req) != 0:
                return req

        return []


class ProvisionRequest(Request):
    def __init__(self, env, monitoring: Monitoring, sla: SLA, price: float) -> None:
        super().__init__()
        self.env = env
        self.price = price
        self.sla = sla

    def __call__(self, node: FogNode, _caller: str) -> Any:
        # yield self.env.timeout(0)
        if node.cores_used + self.sla.core > node.cores:
            return False
        if node.mem_used + self.sla.mem > node.mem:
            return False

        node.cores_used += self.sla.core
        node.mem_used += self.sla.mem

        node.provisioned.append(self.sla)
        monitoring.currently_provisioned += 1
        yield self.env.timeout(self.sla.duration)

        node.cores_used -= self.sla.core
        node.mem_used -= self.sla.mem
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
        pricing_strat: Pricing,
    ):
        self.env = env
        self.network = network
        self.name = name
        self.children: List[FogNode] = []
        self.parent = parent

        self.cores = cores
        self.mem = mem
        self.cores_used = 0.0
        self.mem_used = 0.0
        self.provisioned: List[SLA] = []

        self.pricing_strat = pricing_strat

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


def submit_function(
    env, network, marketplace, function: expe.Function, mon: Monitoring
):
    #  CPU is in millicpu
    sla = SLA(function.mem, function.cpu / 1000, function.latency, 800 * SECS)
    yield env.timeout(function.sleep_before_start * SECS)
    mon.total_submitted += 1
    yield env.process(
        marketplace.auction(function.target_node, EdgeFirstRequest(env, sla, network))
    )


def init_network(env, latencies, node, parent=None, flat_list={}):
    children = node["children"] if "children" in node else []
    fog_node = FogNode(
        env,
        latencies,
        node["name"],
        parent,
        node["flavor"]["reserved_core"],
        node["flavor"]["reserved_mem"],
        ConstantPricing(),
    )
    flat_list[node["name"]] = fog_node

    for child in children:
        child, _ = init_network(env, latencies, child, node, flat_list)
        fog_node.add_children(child)

    return fog_node, flat_list


def generate_latencies(net):
    ret = {}

    def gen_network_cb(source, destination, delay):
        ret[(source, destination)] = delay

    gen_net(net, gen_network_cb)

    return ret


monitoring = Monitoring()

# Setup and start the simulation
print("FaaS Fog")

env = simpy.Environment()

latencies = generate_latencies(NETWORK)
_first_node, network = init_network(env, latencies, NETWORK)

marketplace = MarketPlace(env, network, monitoring)

functions = expe.load_functions(os.getenv("EXPE_SAVE_FILE"))
for function in functions:
    # Use the id of the node instead of its name
    function.target_node = function.target_node.replace("'", "")
    env.process(submit_function(env, latencies, marketplace, function, monitoring))

# Execute!
with alive_bar(
    int(SIM_TIME / SECS), title="Simulating...", ctrl_c=False, dual_line=True
) as bar:
    for ii in range(SIM_TIME):
        if ii % SECS == 0:
            bar.text = f"--> Currently provisioned {monitoring.currently_provisioned},failed to provision {monitoring.total_submitted - monitoring.total_provisioned}, total is {monitoring.total_submitted}..."
            bar()
        env.run(until=1 + ii)


print(
    f"--> Done {monitoring.total_provisioned}, failed to provision {monitoring.total_submitted - monitoring.total_provisioned} functions; total is {monitoring.total_submitted}."
)
