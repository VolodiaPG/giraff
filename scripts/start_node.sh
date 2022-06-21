#!/bin/bash
echo "ID: $1"
ID=$1
env TMP_FILE=$(mktemp)
    OPENFAAS_USERNAME="admin" \
    OPENFAAS_PASSWORD=$(cd ../experiments && poetry run python integration openfaas-login --file $TMP_FILE && cat $TMP_FILE) \
    ROCKET_PORT="300${ID}" \
    OPENFAAS_PORT="8081" \
    NODE_SITUATION_PATH="node-situation-${ID}.ron" \
    cargo run --package manager --bin fog_node
