# About

1. This repository contains a collection of functions ready to be deployed on an OpenFaaS Fog infrastructure.

2. It also is home to a framework that can be used on Fog networks (simple ones for now) to allocate functions using auctions and Service Level Agreements as a way to describe the functions requirements (CPU, RAM, latency, etc.).

> Please consider this project does not have reached alpha version yet and is still under heavy development

# Installation

## Use Grid5000

Most of the deployment process has now been streamlined to Grid5000 using [EnosLib](https://gitlab.inria.fr/discovery/enoslib) as the simplification vector.

As this section is deeply linked with the evaluation process, documentation changes often. Most of the process can be described by reading the CLI help of the scripts living in `node/manager/experiments/`.

Then to create a Grid5000 profile consider using the `init` verb

And to reserve the cluster to then execute use the verb: `up` (respectively `clean` to deallocate and free the nodes).

Another script that builds the docker images used in k8s is available under `/build`.

## Install Locally (with multipass) (deprecated)

### Requirements

- Arkade `curl -SLsf https://get.arkade.dev/ | sudo sh`
- Docker
- Helm `curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash`
- K3sup `curl -sLS https://get.k3sup.dev | sh`
- Kubectl
- Multipass `sudo snap install multipass`

### Installation

Installs K3s
```sh
PUBLIC_SSH_KEY_PATH=$HOME/.ssh/id_ed25519.pub PRIVATE_SSH_KEY_PATH=$HOME/.ssh/id_ed25519 ./scripts/minimal-k3s-multipass-bootstrap.sh
```

To move the kubeconfig file to the local computer config using `kubctl`:
```sh
cp kubeconfig ~/.kube/config
```

> Careful if you have multiple instances running on the same computer

Installs openfaas on the cluster
```sh
arkade install openfaas
```

Install redis on the cluster, backed by longhorn-class storage (you may have to wait a bit to apply redis after the creation of the longhorn storage-class)
```sh
./scripts/longhorn.sh
kubectl apply -f redis
```

### Removal
This is the *__nuke__* option, __finer grained could be better if you have other VMs running__
```sh
multipass delete --all --purge  
```

## Install on Grid'5000 (interactive) (deprecated)

### Connection

`$USER` represents your grid'5000 account name
```sh
ssh $USER@acccess.grid500.fr
ssh rennes
```

### Installation

Get an interactive machine for an hour:
```sh
oarsub "sleep infinity"
oarsub -C $OAR_JOB_ID
```

Enable the use of `sudo` for the rest of the session
```sh
sudo-g5k
```

Install all the good stuff
```sh
curl -sLS https://get.k3sup.dev | sh 
sudo cp k3sup /usr/local/bin/k3sup
export context="k3s-cluster" 
k3sup install --context $context --user $(whoami) --local

curl -SLsf https://get.arkade.dev/ | sudo sh
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
arkade get faas-cli
arkade install openfaas

mv ~/kubeconfig ~/.kube/kubeconfig
export KUBECONFIG=~/.kube/kubeconfig

curl -SLS https://raw.githubusercontent.com/VolodiaPG/fog_application_samples/main/longhorn.sh | bash
svn export https://github.com/volodiapg/fog_application_samples/trunk/redis redis
kubectl apply -f redis
```

### Removal

Killing the provisioned computing space is done via the CLI-frontend
```sh
oardel $OAR_JOB_ID
```

# Usage

## Tunnel connections
To the gateway:
```shell
kubectl port-forward -n openfaas svc/gateway 8080:8080
```

To redis:
```sh
kubectl port-forward -n redis-server svc/redis-server 6379:6379
```

## Login to FaaS CLI
```sh
echo -n $(kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) | faas-cli login --username admin --password-stdin
```

## Other utilities

To overwrite current templates
```sh
faas-cli template pull --overwrite https://github.com/openfaas/templates
```

# Functions

## Up all
```sh
./scripts/all.sh
```
## Architecture

```mermaid
flowchart TD
    A>CarPlate]
    B>Camera image]
    C>IoT weather sensor]
    DB[(Redis)]


    A --> B12[movementplan]

    B --> B1[objectrecognition]
    B1 --> B12
    B1 --> B11[trafficstatistics]
    B1 --> B13[emergencydetection]


    C --> C1[roadcondition] --> E[setlightphasecalculation]

    B11 & B12 & B13 -.- DB
    E -.- DB
```
# Acknowledgments

Inspiration for the functions taken from [BeFaaS](https://github.com/Be-FaaS/BeFaaS-framework)

