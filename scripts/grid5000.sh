#!/bin/bash
curl -sLS https://get.k3sup.dev | sh
sudo cp k3sup /usr/local/bin/k3sup
export context="k3s-cluster"
k3sup install --context $context --user $(whoami) --local
export KUBECONFIG=/home/$(whoami)/kubeconfig
kubectl config set-context $context
kubectl get node -o wide

curl -SLsf https://get.arkade.dev/ | sudo sh
sudo mv /home/voparolguarino/.arkade/bin/faas-cli /usr/local/bin/
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
arkade get faas-cli
#arkade install openfaas
curl -sSL https://cli.openfaas.com | sudo -E sh

# mv ~/kubeconfig ~/.kube/kubeconfig
# export KUBECONFIG=~/.kube/kubeconfig

# curl -SLS https://raw.githubusercontent.com/VolodiaPG/fog_application_samples/main/longhorn.sh | bash
# svn export https://github.com/volodiapg/fog_application_samples/trunk/redis redis
# kubectl apply -f redis
