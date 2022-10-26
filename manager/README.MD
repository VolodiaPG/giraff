# Fog framework

This is the source of:
 - bin `fog_node` framework installed on fog nodes to be used with both k8s & OpenFaaS
 - bin `market` that is responsible for relaying and doing the auctions with the client depositing functions
 - lib `manager` that unites all the common resources (models, OpenFaaS apis, etc.)

# Improvement sectors
- RTT computation
- Protocol, currently only HTTP, maybe useful in the future to use a zero-copy buffer RPC protocol that Rust enables
- Using persistent storage for both the market and the nodes, like [redb](https://github.com/cberner/redb)


TMp:
k3s kubectl exec --namespace openfaas fog-node-8666cf4b8f-wf6vw --container  sidecar-log-agent-fog-node  -it -- /bin/sh
