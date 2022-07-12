_default:
	just --list

# Deploy openfaas and k3s locally. Will open a tunnel in bg
local $nb_instances='1':
	#!/usr/bin/env bash
	./scripts/create_instances.sh
	kubectl rollout status -n openfaas deploy/gateway
	kubectl port-forward -n openfaas svc/gateway 8080:8080 &

clean $instance_i='1':
	multipass delete --purge master-{{instance_i}}
