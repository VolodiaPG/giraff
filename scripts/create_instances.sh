#!/bin/bash
nb_instances="${nb_instances:-2}"

echo "creating 'nb_instances': ${nb_instances}"

# Set up the environement variables
export PUBLIC_SSH_KEY_PATH="${HOME}/.ssh/id_ed25519.pub"
export PRIVATE_SSH_KEY_PATH="${HOME}/.ssh/id_ed25519"

for ii in $(seq 1 "${nb_instances}"); do

	export master="master-${ii}"
	bash -c scripts/minimal-k3s-multipass-bootstrap.sh

	cp kubeconfig "${HOME}/.kube/config"
	export KUBECONFIG="${HOME}/.kube/config"
	mv kubeconfig kubeconfig-master-"${ii}"

	arkade install openfaas

	bash -c scripts/longhorn.sh
	#sleep 30
	#kubectl apply -f redis
done
