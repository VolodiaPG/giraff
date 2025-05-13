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

## About

Function-as-a-Service (FaaS) applications can harness the disseminated nature of
the fog and take advantage of the fog benefits, such as real-time processing and reduced bandwidth. The FaaS programming paradigm allows applications to be divided in independent units called “functions.” However, deciding how to place those units in the fog is challenging. Fog contains diverse, potentially resource-constrained nodes, geographically spanning from the Cloud to the IP network edges. These nodes must be efficiently shared between the multiple applications that will require to use the fog.

We introduce “fog node ownership,” a concept where fog nodes are owned by different actors that chose to give computing resources in exchange for remuneration. This concept allows for the reach of the fog to be dynamically extended without central supervision from a unique decision taker, as currently considered in the literature. For the final user, the fog appears as a single unified FaaS platform. We use auctions to incentivize fog nodes to join and compete for executing functions.

Our auctions let Fog nodes independently put a price on candidate functions to run. It introduces the need of a “Marketplace,” a trusted third party to manage the auctions. Clients wanting to run functions communicate their requirements using Service Level Agreements (SLA) that provide guarantees over allocated resources or the network latency. Those contracts are propagated from the Marketplace to a node and relayed to neighbors.

Key features of Global Integration of Reverse Auctions and Fog Functions (**GIRAFF**):

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
- Python
- Kubernetes (K3S)
- OpenFaaS (do not use in the future)
- EnosLib

## Getting Started

This repo uses extensively [`just`](https://github.com/casey/just) as a powerful CLI facilitator.

.git should be present to allow nix to work properly.

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
2. (optional) Append to /etc/nix/nix.conf:
   ```nix
   extra-experimental-features = nix-command flakes
   max-jobs = auto
   cores = 0
   log-lines = 50
   builders-use-substitutes = true
   trusted-useres = root <YOUR USERNAME>
   ```
   > This enables commands such as `nix develop` without the additional options , multithreading, bigger logs and the usage of the projet's [cachix](https://giraff.cachix.org) cache
3. (optional) install direnv to simplify navigating the project and loading
   dependenies
4. All usual commands can be found in the justfiles, just type `just --list`

> These commands work when `flake.nix` and `justfile` are present in the current directory you are in.

> The following guide has been tested on the 12th of May 2025 on Ubuntu server,
> in Proxmox with KVM acceleration and host cpu. The PC was equipped with a 4th
> gen Intel i7 and 14G of RAM, but has frozen up. Thus, we recommend more
> resources, as we detail in the next section.

### Running the experiments

One should have the `libvirt` daemon (`libvirtd`) running with KVM acceleration.
Make sure your user gets in the `libvirt`, `kvm` groups. You may need to reboot
after installation. The following steps are
going to run multiple VMs using Vagrant to emulate Fog nodes. The setup is much
lower scale than in the real experiments (3ish VMs instead of 663), but still
consumes a lot of RAM (recommended at least 32G and the same amount of swap) and disk space (recommended at
least 100G). Note that there exists a simpler way of running locally for
development.

> Note that most of the steps will take minutes to download and/or build
> and complete.

From the root of the project,

1. cd in `testbed/iso`
2. run `nix develop --extra-experimental-features "nix-command flakes" .#iso`
3. run `just build-vagrant` to create the VMs template to use locally. The
   command should terminate with a message indicating the successful installation
   of the "vagrant box" into `~/.vagrant.d/boxes/giraffbox/0/libvirt/`

From the root of the project,

1. cd into `testbed`
2. run `nix develop --extra-experimental-features "nix-command flakes" .#testbed`
3. run `just master_docker_campaign` to start vagrant and run automatically the
   experiments
4. Logs are available to `tail -f` in the `logs_campaign` directory if running
   anything else than the DEV mode, otherwise they should simply appear. The
   experiment will likely at least run for 30 minutes. We have configured a small 5
   minutes duration for each of the deployment algorithm. In the log you should see
   the reservation of functions happening, and restarts in between each run for
   each placement algorithm.
5. Once finished, results are available in `metrics-arks` directory.
6. You may also copy the name of the `.tar.xz` files printed out in the logs to
   input the next section.

> To remove all trace of the VMs, run `just clean-vagrant`

Currently, Vagrant and Enoslib do not support setting up networking with rate
limiting and adding delays. Thus, these aspects can only be reproduced on Grid5000.

To observe the VMs, one can use the `vagrant ssh ...` command to connect
to a node; the command has to be issued in the `valuations\*` directory for it
to work.
The name of the VMs can be found with the `vagrant global-status` command.

Inside a VM, the command `k9` will open the status of the k3s cluster.
Sometimes, the docker registry may rate limit, but we tried to circumvent this
limitation by hosting our own images on ghcr.io.

> The files `trace_buildvm.txt` and `trace_expe_running.txt` showcase outputs of
> the building of the vm and running of experiments.

### Graphing the results

Once experiments have finished, or with the artifacts of our paper:

1. cd into `testbed/mining`
2. run `nix develop --extra-experimental-features "nix-command flakes" .#mining`
3. From there, 3 separate shells are required for the following commands: `just logs`,
   `just watch`, `just serve`, in that order.
4. To graph the results, edit the `config.R` file and in the `METRICS_ARKS`
   variable, replace the first elements until the comments to paste your results. Format accordingly to R: add a `"` at the beginning of each line, and a `",` at the end of each line.
5. Logs should appear in the two first shells opened, ending with a green `OK`
   message.
6. To access the same graphs put into the paper, open the URL `http://localhost:9000` in a
   browser. There, you may find a list of graphs to interact with, in `htm`
   formats.

Please note that artifacts used to produce the graphs in our articles are
available in the [release section of our
repository](https://github.com/VolodiaPG/giraff/releases/tag/v3.0.0). They would
need to be put in the `metric-arks` directory.

### Tweaking the configuration

The experiments run a cluster of VMs, each connected to another from the
definitions generated in the `definitions.py` NETWORK variable.

Tweaking for the settings of the experiments is done in three files:

- `.env`, this file contains the settings for choosing the different placement
  algorithms, functions to run, and setting about the general configuration of any
  experiment. Importantly, the file also stipulates the different settings for
  making different scenarios. Some settings are also fed to the just commands
  automatically.
- `.experiments.env` contains settings about the maximum size of fog testbed, and
  its size variations can be set here.
- `.env.[1-9]+` files, for example `.env.1`, contain tweaks for a specific scenario (e.g. the number of function to randomly submit to nodes in the fog network)

For each combination of size and number of VMs, a new run will be started. Those
can additionally be run multiple times. In those runs, and for each placement
algorithm, the exact same fog network will be deployed and request replayed. A
restart is performed in between each run of an algorithm to reset the state, as
we employ an impermanent VM configuration.

## Advanced usage

This section concerns OpenFaaS functions, the code for the fog nodes under `manager/`, the code for `iot_emulation`, the code for the proxy.

Usually, `nix develop` gets you started. Using VS Code, thus the extension [Direnv](https://marketplace.visualstudio.com/items?itemName=mkhl.direnv) will make VS Code use all the applications/env loaded in the `nix develop`, e.g. you can use Rust/Golang LSPs server/toolchains inside VS Code without ever “installing” them on the computer.

### VMs

VMs detail are located in `testbed/iso/flake.nix`. There lies the whole configuration of the experimental VM. In `testbed/flake.nix` you can also see the VM used to deploy the code in grid’5000.

To start the same vm as in the experiments, one should go to `testbed/iso` and enter a `nix develop`. Then starting the VM is a matter of `just vm`. Once the VM is started, connection can be made with `just ssh-in`.

> This process is used to start the VM used to develop the fog node software programs

### Experiments

This uses the exact same VMs as the previous section. To generate the VM disk
and send it to grid’5000, enter `just upload <rennes|nancy|...>` from the
testbed/iso directory. The disk will be uploaded.

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

### About included functions

Our own functions are:

- `echo` simply register to Influx the arrival time of the functions. It can transfer the request to a next function. (Rust)

We borrowed functions from [EdgeFaaSBench](https://github.com/kaustubhrajput46/EdgeFaaSBench)

- `speech-to-text` (Python)

### Processing the results

This flake has two modes: `just labExport` will start a JupyterLab with Latex
support and Tikz export for the graphs, `just lab` starts a lighter version
without Latex and Tikz. just watch can be used to keep refreshing the
compilation.

Then, data exploitation is done using R inside the JupyterLab server. just serve
can setup a small server to access interactive plots.

> With article submissions, one can find the raw data in the [Release page](https://github.com/volodiapg/faas_fog/releases/latest). Take the latest, extract it to `testbed` under a directory named `metrics-arks`. Then run `just lab` and you will be able to explore the data. Please notice that this process is heavy on the CPU and especially the RAM. I used some Systemd magic to prevent my computer from using too much RAM and cut the program if so.

### Dev

To locally develop, I will describe the simple steps to get started:

- start the VM: `cd testbed/iso; just vm`
- start the iot_emulation: `cd iot_emulation; just run`
- start the manager (fog node && market): `cd manager; just run <ip:local ip(not localhost)>`
- upload the functions to the registry: `cd openfaas-functions; just`
- you can run parts of the experimental configuration using `cd manager; just expe <ip:same as before>`

> [!TIP]
> On grid5k, on your `~/`, you can paste a tailscale auth token (ephemeral; reusable) in `~/tailscale_authkey` to automatically connect newly spawned VMs to tailnet. That way, you can access Jaeger to see logs from the comfort of your web browser, for example.

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
- TCP latency/throughput estimation: [1] T. J. Hacker, B. D. Athey, et B. Noble, « The end-to-end performance effects of parallel TCP sockets on a lossy wide-area network », dans Proceedings 16th International Parallel and Distributed Processing Symposium, Ft. Lauderdale, FL: IEEE, 2002, p. 10 pp. doi: 10.1109/IPDPS.2002.1015527. and Bolliger, J., Gross, T. and Hengartner, U., Bandwidth modeling for network-aware applications. In INFOCOM '99, March 1999.
