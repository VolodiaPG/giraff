#!/bin/bash
echo "ID: $1"
ID=$1
env KUBECONFIG="../../kubeconfig-master-1" \
    OPENFAAS_USERNAME="admin" \
    OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas --kubeconfig=${KUBECONFIG} basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) \
    ROCKET_PORT="300${ID}" \
    OPENFAAS_PORT="8081" \
    NODE_SITUATION_PATH="node-situation-${ID}.ron" \
    cargo run --package manager --bin fog_node
