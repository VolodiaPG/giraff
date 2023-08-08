from __future__ import annotations
from abc import ABC, abstractmethod
from collections import defaultdict
import csv
from dataclasses import dataclass, field
import functools
import math
import os
import random
import sys
from typing import Any, List, Tuple, Dict
from alive_progress import alive_bar
import numpy
import simpy
from definitions import LEVELS, NETWORK, gen_net
import expe
import scipy.integrate as integrate

MS = 1
SECS = 1000

SIM_TIME = expe.FUNCTION_RESERVATION_FINISHES_AFTER * SECS

RANDOM_SEED = expe.RANDOM_SEED
if RANDOM_SEED is not None and RANDOM_SEED != "":
    random.seed(RANDOM_SEED)


@dataclass
class SLA:
    mem: float
    core: float
    latency: float
    duration: float


@dataclass(order=True)
class Bid:
    bid: float
    bidder: str


@dataclass
class Monitoring:
    currently_provisioned = 0
    provisioned: Dict[str, float] = field(
        default_factory=lambda: defaultdict(lambda: 0.0)
    )
    total_provisioned = 0
    total_submitted = 0
    earnings: Dict[str, List[float]] = field(
        default_factory=lambda: defaultdict(lambda: [])
    )


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
    def __init__(self, price) -> None:
        self.price = price

    def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
        return Bid(self.price, node.name)


class RandomPricing(Pricing):
    def __init__(self, max_price: float) -> None:
        self.max_price = max_price

    def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
        return Bid(random.uniform(0, self.max_price), node.name)


class LinearPricing(Pricing):
    def __init__(
        self,
        slope: float,
        fog_level: int,
        max_level: int,
        max_initial_price: float,
    ) -> None:
        self.slope = slope
        self.initial_price = self.generate_initial_price(
            fog_level,
            max_level,
            max_initial_price,
        )

    def generate_initial_price(
        self,
        location: int,
        max_location: int,
        max_initial_price: float,
    ):
        # Ensure location is within the valid range (1 to max_location)
        location = min(max(1, location), max_location - location)

        # Calculate the base price as an inverse function of location
        # base_price = max_initial_price - (max_initial_price * (location - 1) / (max_location - 1))
        base_price = (
            (1 - max_initial_price) * location + max_location * max_initial_price - 1
        ) / (
            max_location - 1
        )  # to guarantee that price for location=1 is max_price and price for max_location is 1
        # Add random noise to the base price to create variation
        noise = (max_initial_price - 1) / (max_location - 1) * 2  # half of the slope
        random_variation = random.uniform(
            -noise, noise
        )  # Adjust the range of variation as needed

        # Calculate the final price by adding random variation to the base price
        price = base_price + random_variation

        # Ensure the price is within the desired range (1 to max_initial_price )
        price = min(max(1, price), max_initial_price)
        return price

    # def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
    #     u1 = node.cores_used / node.cores
    #     u2 = (node.cores_used + sla.core) / node.cores
    #     integral = (
    #         (u2 - u1) * (self.slope * u2 + self.slope * u1 + 2 * self.initial_price) / 2
    #     )
    #     return Bid(integral, node.name)

    def price(self, utilization):
        # return self.slope * utilization
        return self.initial_price + self.slope * utilization

    def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
        price, _ = integrate.quad(
            self.price,
            (node.cores_used) / node.cores,
            (node.cores_used + sla.core) / node.cores,
        )
        return Bid(price, node.name)


class LinearPerPartPricing(Pricing):
    def __init__(
        self,
        slopes: List[float],
        breaking_points: List[float],
        fog_level: int,
        max_level: int,
        max_initial_price: float,
    ) -> None:
        assert len(slopes) == len(breaking_points) + 1
        assert len(breaking_points) != 0
        self.slopes = slopes
        self.breaking_points = breaking_points
        self.initial_price = self.generate_initial_price(
            fog_level,
            max_level,
            max_initial_price,
        )

    def generate_initial_price(
        self,
        location: int,
        max_location: int,
        max_initial_price: float,
    ):
        # Ensure location is within the valid range (1 to max_location)
        location = min(max(1, location), max_location - location)

        # Calculate the base price as an inverse function of location
        # base_price = max_initial_price - (max_initial_price * (location - 1) / (max_location - 1))
        base_price = (
            (1 - max_initial_price) * location + max_location * max_initial_price - 1
        ) / (
            max_location - 1
        )  # to guarantee that price for location=1 is max_price and price for max_location is 1
        # Add random noise to the base price to create variation
        noise = (max_initial_price - 1) / (max_location - 1) * 2  # half of the slope
        random_variation = random.uniform(
            -noise, noise
        )  # Adjust the range of variation as needed

        # Calculate the final price by adding random variation to the base price
        price = base_price + random_variation

        # Ensure the price is within the desired range (1 to max_initial_price )
        price = min(max(1, price), max_initial_price)
        return price

    def price(self, utilization):
        slope_ii = 0
        for ii, break_point in enumerate(self.breaking_points):
            if utilization <= break_point:
                break
            slope_ii = ii + 1
        # return self.slopes[slope_ii] * utilization
        return self.initial_price + self.slopes[slope_ii] * utilization

    def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
        price, _ = integrate.quad(
            self.price,
            node.cores_used / node.cores,
            (node.cores_used + sla.core) / node.cores,
        )
        return Bid(price, node.name)


class LogisticPricing(Pricing):
    def __init__(
        self, A: float, K: float, C: float, Q: float, V: float, B: float
    ) -> None:
        self.A = A
        self.K = K
        self.C = C
        self.Q = Q
        self.V = V
        self.B = B

    def price(self, utilization):
        return self.A + (self.K - self.A) / (
            math.pow(
                self.C + self.Q * math.exp(-1.0 * self.B * utilization), 1 / self.V
            )
        )

    def __call__(self, node: FogNode, sla: SLA, accumulated_latency: float) -> Bid:
        price, _ = integrate.quad(
            self.price,
            node.cores_used / node.cores,
            (node.cores_used + sla.core) / node.cores,
        )
        return Bid(price, node.name)


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
    node: FogNode
    latency: float


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
            if delay + self.acc_latency > self.sla.latency:
                continue
            nodes.append(SortableFogNode(child, self.acc_latency + delay))

        nodes = sorted(nodes, key=lambda x: x.latency)

        for nodepack in nodes:
            node = nodepack.node
            delay = nodepack.latency
            req = yield env.process(
                node.send(
                    child,
                    EdgeFirstRequest(
                        self.env,
                        self.sla,
                        self.network,
                        delay,
                    ),
                )
            )
            if len(req) != 0:
                return req

        return []


@dataclass
class EdgeFirstRequestData:
    distance: float
    bid: Bid


class EdgeFirstRequestTwo(Request):
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
        # yield self.env.timeout(0)  # Make the method a generator
        ret = yield self.env.process(self.call_inner(node, caller))
        if caller == "":
            return [ret.bid] if ret is not None else []
        return ret

    def call_inner(self, node: FogNode, caller: str):
        if (
            node.cores_used + self.sla.core <= node.cores
            and node.mem_used + self.sla.mem <= node.mem
        ):
            return EdgeFirstRequestData(
                self.acc_latency, node.pricing_strat(node, self.sla, self.acc_latency)
            )

        nodes: Any = []
        for child in node.children + [node.parent] if node.parent is not None else []:
            if child.name == caller:
                continue
            delay = self.network[(node.name, child.name)]
            if delay + self.acc_latency > self.sla.latency:
                continue
            nodes.append(SortableFogNode(child, self.acc_latency + delay))

        nodes = sorted(nodes, key=lambda x: x.latency)

        for ii, nodepack in enumerate(nodes):
            node = nodepack.node
            delay = nodepack.latency
            req = yield env.process(
                node.send(
                    child,
                    EdgeFirstRequestTwo(
                        self.env,
                        self.sla,
                        self.network,
                        delay,
                    ),
                )
            )
            if req is None:
                continue
            if ii == len(nodes) - 1:
                return req
            if req.distance < nodes[ii + 1].latency:
                return req

        return None


@dataclass
class FurthestPlacementRequestData:
    rank: float
    bid: Bid


class FurthestPlacementRequest(Request):
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
                        FurthestPlacementRequest(
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
            bids.append(
                FurthestPlacementRequestData(
                    LEVELS.get(node.name),
                    node.pricing_strat(node, self.sla, self.acc_latency),
                )
            )

        if caller == "":  # First call
            bids = sorted(bids, key=lambda x: x.rank)
            return [bids[0].bid] if len(bids) > 0 else []

        return bids


class ProvisionRequest(Request):
    def __init__(self, env, monitoring: Monitoring, sla: SLA, price: float) -> None:
        super().__init__()
        self.env = env
        self.price = price
        self.sla = sla
        self.monitoring = monitoring

    def __call__(self, node: FogNode, _caller: str) -> Any:
        # yield self.env.timeout(0)
        if node.cores_used + self.sla.core > node.cores:
            return False
        if node.mem_used + self.sla.mem > node.mem:
            return False

        node.cores_used += self.sla.core
        node.mem_used += self.sla.mem
        node.provisioned.append(self.sla)

        self.monitoring.provisioned[node.name] += 1
        self.monitoring.earnings[node.name].append(self.price)
        self.monitoring.currently_provisioned += 1
        self.monitoring.total_provisioned += 1

        yield self.env.timeout(self.sla.duration)

        node.cores_used -= self.sla.core
        node.mem_used -= self.sla.mem
        node.provisioned.remove(self.sla)

        self.monitoring.currently_provisioned -= 1
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

    def auction(self, first_node: str, sla: SLA, auction: Request):
        bids: List[Bid] = yield self.env.process(self.send(first_node, auction))
        if len(bids) == 0:
            return

        bids = sorted(bids, key=lambda x: x.bid)

        winner = bids[0]
        price = bids[1].bid if len(bids) > 1 else bids[0].bid
        # print(winner.bidder)
        ret = yield self.env.process(
            self.send(
                winner.bidder,
                ProvisionRequest(env, self.monitoring, sla, price),
            )
        )
        return ret


def submit_function(
    env, network, marketplace, function: expe.Function, mon: Monitoring, strat_type
):
    #  CPU is in millicpu
    sla = SLA(function.mem, function.cpu / 1000, function.latency * MS, 800 * SECS)
    yield env.timeout(function.sleep_before_start * SECS)
    mon.total_submitted += 1
    yield env.process(
        marketplace.auction(function.target_node, sla, strat_type(env, sla, network))
    )


def init_network(
    env, latencies, node, pricing_strategy, parent=None, flat_list={}, level=0
):
    children = node["children"] if "children" in node else []
    fog_node = FogNode(
        env,
        latencies,
        node["name"],
        parent,
        node["flavor"]["reserved_core"],
        node["flavor"]["reserved_mem"],
        # LinearPricing(2.0, level, 3, 20.0),
        pricing_strategy(level + 1),
    )
    flat_list[node["name"]] = fog_node

    for child in children:
        child, _ = init_network(
            env, latencies, child, pricing_strategy, node, flat_list, level + 1
        )
        fog_node.add_children(child)

    return fog_node, flat_list


def generate_latencies(net):
    ret = {}

    def gen_network_cb(source, destination, delay):
        ret[(source, destination)] = delay

    gen_net(net, gen_network_cb)

    return ret


def choose_from(env_var, mapping):
    key = os.getenv(env_var, "")
    ret = mapping.get(key)
    if ret is None:
        print(f"{env_var} not in [{' '.join(list(mapping.keys()))}]")
    else:
        print(f"Using {env_var} {key} -> {ret}", file=sys.stderr)
    return ret, key


monitoring = Monitoring()

# Setup and start the simulation
max_level = 0
for level in LEVELS.values():
    max_level = max(max_level, level)

placement_strategy, placement_strategy_name = choose_from(
    "PLACEMENT_STRATEGY",
    {
        "auction": AuctionBidRequest,
        "edge_first": EdgeFirstRequest,
        "edge_first2": EdgeFirstRequestTwo,
        "edge_ward": EdgeWardRequest,
        "furthest": FurthestPlacementRequest,
    },
)

pricing_strategy, pricing_strategy_name = choose_from(
    "PRICING_STRATEGY",
    {
        "same": functools.partial(lambda _: ConstantPricing(1.0)),
        "constant": ConstantPricing,
        "random": functools.partial(lambda _: RandomPricing(10.0)),
        "linear": functools.partial(
            lambda level: LinearPricing(8.0, level, max_level, 10.0)
        ),
        "linear_part": functools.partial(
            lambda level: LinearPerPartPricing(
                [1.0, 2.0, 8.0], [0.2, 0.5], level, max_level, 10.0
            )
        ),
        "logistic": functools.partial(lambda _: LogisticPricing(0, 1, 1, 10, 0.9, 5)),
        "logistic_inv": functools.partial(
            lambda _: LogisticPricing(1, 0, 1, 10, 0.9, 5)
        ),
    },
)

if not pricing_strategy or not placement_strategy:
    exit(1)

env = simpy.Environment()

latencies = generate_latencies(NETWORK)
_first_node, network = init_network(env, latencies, NETWORK, pricing_strategy)

marketplace = MarketPlace(env, network, monitoring)

functions = expe.load_functions(os.getenv("EXPE_SAVE_FILE"))
for function in functions:
    # Use the id of the node instead of its name
    function.target_node = function.target_node.replace("'", "")
    env.process(
        submit_function(
            env, latencies, marketplace, function, monitoring, placement_strategy
        )
    )

# Execute!
with alive_bar(
    SIM_TIME, title="Simulating...", ctrl_c=False, dual_line=True, file=sys.stderr
) as bar:
    for ii in range(SIM_TIME):
        if ii % (SECS) == 0:
            bar.text = f"--> Currently provisioned {monitoring.currently_provisioned},failed to provision {monitoring.total_submitted - monitoring.total_provisioned}, total is {monitoring.total_submitted}..."
            bar(SECS)
        env.run(until=1 + ii)


print(
    f"--> Done {monitoring.total_provisioned}, failed to provision {monitoring.total_submitted - monitoring.total_provisioned} functions; total is {monitoring.total_submitted}.",
    file=sys.stderr,
)

earnings: List[List[List[float]]] = [[] for _ in range(max_level + 1)]
provisioned: List[List[float]] = [[] for _ in range(max_level + 1)]
count = [0] * (max_level + 1)

for node, level in LEVELS.items():
    earnings[level].append(monitoring.earnings[node])
    provisioned[level].append(monitoring.provisioned[node])
    count[level] += 1


def quantile(x, q):
    if len(x) == 0:
        return float("nan")
    return numpy.quantile(x, q)


for ii in range(max_level + 1):
    earn = functools.reduce(lambda x, y: x + y, earnings[ii])
    prov = provisioned[ii]
    print(
        f"""Lvl {ii} ({count[ii]} nodes):
    Earnings: tot: {numpy.sum(earn):.2f} med: {numpy.median(earn):.2f} [{quantile(earn, .025):.2f},{quantile(earn, .975):.2f}] avg: {numpy.mean(earn):.2f}
    Provisioned: tot: {numpy.sum(prov)} med: {numpy.median(prov):.2f} [{quantile(prov, .025):.2f},{quantile(prov, .975):.2f}] avg: {numpy.mean(prov):.2f}""",
        file=sys.stderr,
    )

seed = RANDOM_SEED or int(-1)
writer = csv.writer(sys.stdout, delimiter="\t")
if (os.getenv("JOB_INDEX", "1")) == "1":
    writer.writerow(
        [
            "job_id",
            "node",
            "placement",
            "pricing",
            "seed",
            "level",
            "earning",
            "provisioned",
        ]
    )
for node, level in LEVELS.items():
    for cost in monitoring.earnings[node]:
        writer.writerow(
            [
                os.getenv("JOB_INDEX", "1"),
                node,
                placement_strategy_name,
                pricing_strategy_name,
                seed,
                level,
                cost,
                monitoring.provisioned[node],
            ]
        )
