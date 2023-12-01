
<h1 align="center">
  <a href="https://github.com/volodiapg/giraff">
    <img src="giraffe.jpg" alt="GIRAFF mascot" width="320" height="320" style="border-radius: 50%;box-shadow: rgba(50, 50, 93, 0.25) 0px 30px 60px -12px, rgba(0, 0, 0, 0.3) 0px 18px 36px -18px;">
  </a>
</h1>

<div align="center">
  <h1>
    GIRAFF: Reverse auction-based placement for fog functions 
  </h1>
  <h3>
  Using FaaS functions and auctions for a sustainable fog model
  </h3>
  <!-- <br />
  <br />
  <a href="">Report a Bug</a>
  ·
  <a href="">Request a Feature</a>
  .
  <a href="">Ask a Question</a> -->
</div>

<div align="center">
<br />

[![license](https://img.shields.io/badge/License-MIT-success?style=flat-square)](LICENSE)

[![made with hearth by VolodiaPG](https://img.shields.io/badge/Made%20with%20%E2%99%A5%20by-VolodiaPG-%23ff0000?style=flat-square)](https://github.com/volodiapg)

</div>

<details open="open">
<summary>Table of Contents</summary>

- [About](#about)
  - [Built With](#built-with)
- [Getting Started](#getting-started)
  - [Overview](#overview)
  - [Prerequisites](#prerequisites)
  - [Software](#software)
  - [VMs](#vms)
  - [Experiments](#experiments)
  - [Processing the results](#processing-the-results)
  - [Notes about the system architectures](#notes-about-the-system-architectures)
- [Contribution or utilization](#contribution-or-utilization)
- [License](#license)
- [Acknowledgements](#acknowledgements)

</details>

## About

Function-as-a-Service (FaaS) applications could harness the disseminated nature of the fog and take advantage of fog’s benefits, such as real-time processing and reduced bandwidth. The FaaS programming paradigm allows applications to be divided in independent units called “functions.” However, deciding how to place those units in the fog is challenging. Fog contains diverse, potentially resource-constrained nodes, geographically spanning from the Cloud to the IP network edges. These nodes must be efficiently shared between the multiple applications that will require to use the fog.

We introduce “fog node ownership,” a concept where fog nodes are owned by different actors that chose to give computing resources in exchange for remuneration. This concept allows for the reach of the fog to be dynamically extended without central supervision from a unique decision taker, as currently considered in the literature. For the final user, the fog appears as a single unified FaaS platform. We use auctions to incentivize fog nodes to join and compete for executing functions.

Our auctions let Fog nodes independently put a price on candidate functions to run. It introduces the need of a “Marketplace,” a trusted third party to manage the auctions. Clients wanting to run functions communicate their requirements using Service Level Agreements (SLA) that provide guarantees over allocated resources or the network latency. Those contracts are propagated from the Marketplace to a node and relayed to neighbors.

Key features of Global Integration of Reverse Auctions and Fog Functions (**GIRAFF**):

- Standard stack (Kubernetes, OpenFaaS)
- Efficient programming in Rust to enable effective collaboration (also cross-compiling in the future?)
- Nix to reproduce scientifically the experiments and maintain the same development environment for everyone
- Functions to deploy
- [Grid’5000](https://www.grid5000.fr/w/Grid5000:Home) support thanks to [EnosLib](https://discovery.gitlabpages.inria.fr/enoslib/)

<details>
<summary>Additional info</summary>
<br>

This project has been started as an internship sponsored as I was a student from National Institute of Applied Sciences Rennes (INSA Rennes) and a second master (SIF) under University of Rennes 1, University of Southern Brittany (UBS), ENS Rennes, INSA Rennes and CentraleSupélec.

A thesis is being financed by the «Centre INRIA de l’Université de Rennes» to pursue the work.

</details>

### Built With

- Nix
- Rust
- Go
- Kubernetes (K3S)
- OpenFaaS
- EnosLib

## Getting Started

This repo uses extensively [`just`](https://github.com/casey/just) as a powerful CLI facilitator.

### Overview

Here is an overview of the content of this repo:

```shell
.
├── testbed # Contains EnosLib code to interact with Grid'5000 (build + deployment of live environment at true scale)
├── manager # contains the code of the marketplace and the fog_node
├── iot_emulation # Sends request to fog nodes in the experiments to measure their response time, etc.
└── openfaas-functions # contains code of Fog FaaS functions
```

### Prerequisites

1. Install [Nix](https://nixos.org/download#download-nix)
    > [Nix portable](https://github.com/DavHau/nix-portable) is not officially supported, however it may be enough to explore the project without installing Nix
2. Append to /etc/nix/nix.conf:
    ```nix
    extra-experimental-features = nix-command flakes
    max-jobs = auto
    cores = 0
    log-lines = 50
    builders-use-substitutes = true
    trusted-useres = root <YOUR USERNAME>
    ```
    > This enables commands such as `nix develop`, multithreading, bigger logs and the usage of the projet's [cachix](https://giraff.cachix.org) cache
3. `nix develop` (or configure direnv)
4. All usual commands can be found in the justfiles, just type `just --list`

> These commands work when `flake.nix` and `justfile` are present in the current directory you are in.

### Software

This section concerns OpenFaaS functions, the code for the fog nodes under `manager/`, the code for `iot_emulation`, the code for the proxy.

Usually, `nix develop` gets you started. I code using VS Code, thus the extension [Direnv](https://marketplace.visualstudio.com/items?itemName=mkhl.direnv) will make VS Code use all the applications/env loaded in the `nix develop`, e.g. you can use Rust/Golang LSPs server/toolchains inside VS Code without ever “installing” them on the computer.

### VMs

VMs detail are located in `testbed/iso/flake.nix`. There lies the whole configuration of the experimental VM. In `testbed/flake.nix` you can also see the VM used to deploy the code in grid’5000. 

To start the same vm as in the experiments, one should go to `testbed/iso` and enter a `nix develop`. Then starting the VM is a matter of `just vm`. Once the VM is started, connection can be made with `just ssh-in`.
> This process is used to start the VM used to develop the fog node software programs

### Experiments

This uses the exact same VMs as the previous section. To generate the VM disk and send it to grid’5000, enter `just upload <rennes|nancy|...>`. The disk will be uploaded.

Then go to `testbed`, once this is done, you can configure the experiments in `.env`; `.experiments.env` handles the variations of multiple runs. These values are used in `integration.py` that handles the deployment and `definitions.py` that defines the fog network and some Kubernetes configurations.

Then, `just upload` will rsync both the master’s VM and the previously described files to the configured grid’5000 cluster.

Finally, `just master_exec <ghcr.io username> <experiment name>` will start the experiment as configured. Do not forget to make the ghcr.io image public. The different parts of GIRAFF make use of labels to only have 1 image public.

> Note that in `.experiments.env` there are some options to gracefully handle failures, as I use GNU parallel, one can “resume” a job that had some failures before.

In the end, experimental results will be available on the cluster in `metric-arks`. One is able to get them back using the command `just get_metrics_back`. This will download them in the local `metrics-arks` folder.

#### Relation to Azure traces

Azure FaaS traces have been released in 2020. Those traces have been characterized by probabilistic laws [[Hernod](https://doi.org/10.1145/3542929.3563468)]. Those laws are described in the following:

- Execution times follow a highly-variable Log-normal law;
- functions live in the range of milliseconds to minutes; 
- ~~Functions are billed with a millisecond granularity;~~
- the median execution time if 600ms;
- the 99%th execution time is more than 140 seconds;
- 0.6% of functions account for 90% of total invocations (they also test a multiple functions balanced workload, where each function receives the same load);
- arrivals follow an open-loop Poisson law;
- arrival burstiness index is -0.26;
- total number of invocations does not vary much;
- invocations follow diurnal patterns as the Cloud does too;
- functions are busy-spun for the execution_duration, repeating a timed math operation over-and-over.

In their simulations/experiments, they state they use:

- mu = -0.38
- sigma = 2.36

For their experiments, they use MSTrace and select a subset of real traces to be re-run on the FaaS platform.
Functions are 256MB of RAM.

#### Distribution for fog specific characteristics

- A Student T distribution is used for function latencies (as we don't know the real sample size nor the std deviation). The distribution would be divided in three buckets: functions that do not care about the latency (meaning the threshold t is at 10 secs), functions that are normal: they would want their responses back in a usable time (t = 150ms), and low latency functions (t = 15ms). The two extrema would be 5% of the total number functions. We use df=10 with -2.25 and 2.25 for the extrema.

### Included functions

Our own functions are:
  
  - `echo` simply register to Influx the arrival time of the functions. It can transfer the request to a next function. (Rust)

We borrowed functions from [EdgeFaaSBench](https://github.com/kaustubhrajput46/EdgeFaaSBench)
  
  - `speech-to-text` (Python)

### Processing the results

This flake has two modes: `just labExport` will start a JupyterLab with Latex support and Tikz export for the graphs, `just lab` starts a lighter version without Latex and Tikz.

Then, data exploitation is done using R inside the JupyterLab server.

> With article submissions, one can find the raw data in the [Release page](https://github.com/volodiapg/faas_fog/releases/latest). Take the latest, extract it to `testbed` under a directory named `metrics-arks`.  Then run `just lab` and you will be able to explore the data. Please notice that this process is heavy on the CPU and especially the RAM. I used some Systemd magic to prevent my computer from using too much RAM and cut the program if so.

### Dev

To locally develop, I will describe the simple steps to get started:
- start the VM: `cd testbed/iso; just vm`
- start the iot_emulation: `cd iot_emulation; just run`
- start the manager (fog node && market): `cd manager; just run <ip:local ip(not localhost)>`
- upload the functions to the registry: `cd openfaas-functions; just`
- you can run parts of the experimental configuration using `cd manager; just expe <ip:same as before>`


### Notes about the system architectures

Most of the flakes are compatible with both Linux and macOS. However, when generating packages for Linux (like the VM), only Linux can. Extension could be done to enable full cross-platform support.

## Contribution or utilization

Please open an issue or contact me from the info in my [GitHub profile](https://github.com/volodiapg) so that I may be of assistance.

## License

This project is licensed under the **MIT license**.

See [LICENSE](LICENSE) for more information.

## Acknowledgements

Thanks for these awesome resources that were used during the development of this project

- [GNU Parallel](https://www.gnu.org/software/parallel/)
- [EnosLib](https://discovery.gitlabpages.inria.fr/enoslib/index.html)
- [Grid’5000](https://www.grid5000.fr/w/Grid5000:Home)
