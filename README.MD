
<!-- <h1 align="center">
  <a href="https://github.com/volodiapg/faas_fog">
    <img src="{{cookiecutter.repo_slug}}/docs/images/logo.svg" alt="Logo" width="125" height="125">
  </a>
</h1> -->

<div align="center">
  <h1>
    FaaS Fog — Using functions and auctions for a sustainable Fog model
  </h1>
  <h2>
  Function placement for FaaS applications in the Fog​
  </h2>
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
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

</details>

---

## About

<table>
<tr>
<td>

Function-as-a-Service (FaaS) applications could harness the disseminated nature of the Fog and take advantage of Fog’s benefits, such as real-time processing and reduced bandwidth. The FaaS programming paradigm allows applications to be divided in independent units called “functions.” However, deciding how to place those units in the Fog is challenging. Fog contains diverse, potentially resource-constrained nodes, geographically spanning from Cloud to the network edges. These nodes must be efficiently shared between the multiple applications that will require to use the Fog.

We introduce “Fog node ownership,” a concept where Fog nodes are owned by different actors that chose to give computing resources in exchange for remuneration. This concept allows for the reach of the Fog to be dynamically extended without central supervision from a unique decision taker, as currently considered in the literature. For the final user, the Fog appears as a single unified FaaS platform. We use auctions to incentivize Fog nodes to join and compete for executing functions.


Our auctions let Fog nodes independently put a price on candidate functions to run. It introduces the need of a “Marketplace,” a trusted third party to manage the auctions. Clients wanting to run functions communicate their requirements using Service Level Agreements (SLA) that provide guarantees over allocated resources or the network latency. Those contracts are propagated from the Marketplace to a node and relayed to neighbors.

Key features of **our work**:

- Standard stack (Kubernetes, OpenFaaS)
- Efficient programming in Rust for less bugs and to enable effective collaboration (also cross-compiling in the future?)
- Nix to reproduce both scientifically the experiments and maintain the same development environment for everyone
- Functions to deploy
- [Grid'5000](https://www.grid5000.fr/w/Grid5000:Home) support thanks to [EnosLib](https://discovery.gitlabpages.inria.fr/enoslib/)

<details open>
<summary>Additional info</summary>
<br>

This project has been started as an internship sponsored as I was a student from National Institute of Applied Sciences Rennes (INSA Rennes) and a second master (SIF) under University of Rennes 1, University of Southern Brittany (UBS), ENS Rennes, INSA Rennes and CentraleSupélec.

A thesis is being financed by the « Centre INRIA de l'Université de Rennes » to pursue the work.

</details>

</td>
</tr>
</table>

### Built With

- Rust
- Kubernetes (K3S)
- OpenFaaS
- EnosLib
- Nix

## Getting Started

This repo uses extensively [`just`](https://github.com/casey/just) as a powerful CLI facilitator.

### Overview

Here is an overview of the content of this repo:

```shell
.
├── enos # Contain EnosLib code to interact with Grid'5000 (build + deployment of live environment at true scale)
├── manager # contain the code of the marketplace and the fog_node
├── iot_emulation # Sends request to fog nodes in the experiments to measure their response time, etc.
└── openfaas-functions # contain code of Fog FaaS functions
```

### Prerequisites

1. Install Nix
2. `nix develop` (or configure direnv)
3. All usual commands can be found in the justfiles, just type `just --list`

> These commands work when `flake.nix` and `justfile` are present in the current directory you are in.

## Contributing

Please contact me from the info in my [GitHub profile](https://github.com/volodiapg).

## License

This project is licensed under the **MIT license**.

See [LICENSE](LICENSE) for more information.

## Acknowledgements

Thanks for these awesome resources that were used during the development of this project

- for the functions taken from [BeFaaS](https://github.com/Be-FaaS/BeFaaS-framework)
