# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## 0.8.0 - 2023-01-30
#### Bug Fixes
- **(all)** Call faas instead of proxying - (31e153f) - Volodia PAROL-GUARINO
- **(enoslib)** Fix `just refresh` still having passed data in prometheus - (129913c) - Volodia PAROL-GUARINO
- **(enoslib)** Fix wrong ip being used in translation of names - (e3c5f6b) - Volodia PAROL-GUARINO
- **(enoslib)** Fix prometheus listenning on wrong ports - (43cfd45) - Volodia PAROL-GUARINO
- **(enoslib)** Fix lantencies not registered with the iot_emulation service - (1ac97c8) - Volodia PAROL-GUARINO
- **(enoslib)** Fix latencies between fog nodes - (2e23330) - Volodia PAROL-GUARINO
- **(fog_node)** Improve latency estimation calculation - (fac1cec) - Volodia PAROL-GUARINO
- **(fog_node)** Fix auction computation - (d7441e1) - Volodia PAROL-GUARINO
- **(fog_node)** Fix `edge_first` deployment errors blocking all deployment - (bc95fb4) - Volodia PAROL-GUARINO
- **(fog_node)** Fix node not making difference between external and internal openfaas ports - (4dfba68) - Volodia PAROL-GUARINO
- **(fog_node)** Fix register child node not doing the last hop - (498445b) - Volodia PAROL-GUARINO
- **(iot_emulation)** Fix timer going infinite w/ 1 job - (e2dde32) - Volodia PAROL-GUARINO
- **(market)** Add metrics collection route - (783233f) - Volodia PAROL-GUARINO
- **(market)** Fix url missing http:// - (d901744) - Volodia PAROL-GUARINO
- Improve latency, netem and IP printing - (8b98600) - Volodia PAROL-GUARINO
#### Features
- **(enoslib)** Add graph for deployment times - (9ad1e4d) - Volodia PAROL-GUARINO
- **(enoslib)** Better graphs - (cb4bfe2) - Volodia PAROL-GUARINO
- **(enoslib)** Improve recordings and experiment parameters - (86e8a21) - Volodia PAROL-GUARINO
- **(enoslib)** Add model comparison, RMSE and SLA break graphs - (7cfa8c0) - Volodia PAROL-GUARINO
- **(enoslib)** Exploit the enable/disable feature of the CRON of iot_emulation to ensure messages are received before sending new - (21681c9) - Volodia PAROL-GUARINO
- **(enoslib)** Add new cases for the scenario - (815f943) - Volodia PAROL-GUARINO
- **(enoslib)** Add refresh of entire k3s cluster - (e85fd69) - Volodia PAROL-GUARINO
- **(enoslib)** Add a scenario verb for experiments - (d203b46) - Volodia PAROL-GUARINO
- **(iot_emulation)** Add enable/disable the cron jobs - (b9ec169) - Volodia PAROL-GUARINO
- **(iot_emulation)** Add build args to Dockerfile - (8046611) - Volodia PAROL-GUARINO
- **(jupyter)** Add ECDF - (53c928d) - Volodia PAROL-GUARINO
- **(market)** Add metrics for function deployment time - (650da73) - Volodia PAROL-GUARINO
- Add prometheus to local deployment - (5ca1dd2) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- **(enoslib)** Reduce the size of the experiment deployment - (ecb3f88) - Volodia PAROL-GUARINO
- **(enoslib)** Add entries to gitignore - (c6aef88) - Volodia PAROL-GUARINO
- **(enoslib)** Remove old prometheus.yml file from local host - (f5d6f6f) - Volodia PAROL-GUARINO
- **(gitignore)** Increase recording capacity and test accuracy - (91b3a80) - Volodia PAROL-GUARINO
- **(iot_emulation)** Remove useless log - (86abe7e) - Volodia PAROL-GUARINO
- **(lock)** Update - (208899f) - Volodia PAROL-GUARINO
- **(manager)** Update k3d debug setup - (30406f1) - Volodia PAROL-GUARINO
- **(manager)** Add clippy - (1a5767b) - Volodia PAROL-GUARINO
- **(manager)** Clean devcontainer - (d3110e6) - Volodia PAROL-GUARINO
- **(market)** Rename old error - (0216955) - Volodia PAROL-GUARINO
- **(market)** Set other logs to warn - (bd2f793) - Volodia PAROL-GUARINO
- **(openfaas-functions)** Configure VSCode settings for openfaas-functions - (0b8c21d) - Volodia PAROL-GUARINO
- **(openfaas_function)** Clean unused functions - (1725c81) - Volodia PAROL-GUARINO
- **(openfaas_functions)** Update echo function to actix usage - (ca0d76d) - Volodia PAROL-GUARINO
- **(rust)** Update toolchain version - (b064906) - Volodia PAROL-GUARINO
- Reduce the size of deployed functions - (2e09368) - Volodia PAROL-GUARINO
#### Refactoring
- **(enoslib)** Change deployment script to optimally deploy on `edge_first` scenario - (f7c48b5) - Volodia PAROL-GUARINO
- **(fog_node)** Run clippy - (cb1ece2) - Volodia PAROL-GUARINO
- **(fog_node)** Remove routing - (04a038b) - Volodia PAROL-GUARINO
- **(fog_node)** Remove faas routing responsibilities - (47f0774) - Volodia PAROL-GUARINO
- **(iot_emulation)** Remove spawning a new task only for registring prometheus metrics - (60cb519) - Volodia PAROL-GUARINO
- **(justfile)** Change default behavior to list - (ed5f63b) - Volodia PAROL-GUARINO
- **(market)** Remove routing logic - (9950447) - Volodia PAROL-GUARINO
- **(market)** Return ip of node where function has been provisioned - (2ef3c92) - Volodia PAROL-GUARINO
- **(model)** Remove RPC port - (149ef51) - Volodia PAROL-GUARINO

- - -

## 0.7.0 - 2023-01-12
#### Bug Fixes
- **(git)** Hide ark files - (92c2a09) - Volodia PAROL-GUARINO
#### Features
- **(fog_node)** Change healt hroute pings from HEAD to GET requests - (35892a2) - Volodia PAROL-GUARINO
- **(g5k)** Add network and htop monitors and increase iot_emulation resources - (3d59c51) - Volodia PAROL-GUARINO
- **(iot_emulation)** Add jaeger flag - (d842a52) - Volodia PAROL-GUARINO
- **(manager)** Enable full LTO & more optimizations - (8149b06) - Volodia PAROL-GUARINO
- **(results)** Add SLA violation graphs - (60350c0) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- Remove deprecated and old stuff - (43f4da8) - Volodia PAROL-GUARINO
#### Refactoring
- **(experiment)** Change parameters - (3490265) - Volodia PAROL-GUARINO

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).