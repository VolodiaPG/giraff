#!/bin/bash
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
