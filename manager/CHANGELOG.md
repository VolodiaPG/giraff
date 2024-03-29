# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## 0.10.0 - 2023-02-28
#### Features
- Add support for pre commits checks - (2d1634c) - Volodia PAROL-GUARINO
- Use nix as devenv - (ad2f558) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- **(functions)** Update cargo lock - (7fc0111) - Volodia PAROL-GUARINO
- **(gitignore)** Ignore pre commit config files - (c521d91) - Volodia PAROL-GUARINO
- **(manager)** Remove unused dependencies - (414a656) - Volodia PAROL-GUARINO
- enable statix for pre-commit - (c59041e) - Volodia PAROL-GUARINO
- Replace formatter by alejandra and remove statix - (288c318) - Volodia PAROL-GUARINO
- Remove devcontainers - (ec9a0ad) - Volodia PAROL-GUARINO

- - -

## 0.9.0 - 2023-02-27
#### Bug Fixes
- **(experiment)** Fix certificate corruption on g5k - (23caa9d) - Volodia PAROL-GUARINO
- **(experiment)** Add marketplace as a viable alias - (9bdcaa3) - Volodia PAROL-GUARINO
- **(experiment)** Fix mistakenly replaced occurrences in the JSON - (ee2b9d8) - Volodia PAROL-GUARINO
- **(experiment)** Avoid errors if tar is not present on the remote system - (103f938) - Volodia PAROL-GUARINO
- **(experiment)** Increase waiting time before starting sending functions to have time to log the ram&cpu - (a58699f) - Volodia PAROL-GUARINO
- **(experiment)** Fix prometheus historgam - (519951d) - Volodia PAROL-GUARINO
- **(fog_node)** Enable back memory and cpu tracking inside a node - (d4d5a4e) - Volodia PAROL-GUARINO
- **(results)** Fix deployment time metrics - (1b78b79) - Volodia PAROL-GUARINO
- **(vm-disk)** Fix vm launch script - (a93f6fc) - Volodia PAROL-GUARINO
- Fix jupyter lab not starting because previous instance still exists - (72252e4) - Volodia PAROL-GUARINO
#### Features
- **(echo_function)** Use SLA latency in function's metrics - (08f2377) - Volodia PAROL-GUARINO
- **(echo_function)** Use the SLA latency as the base for a relevant histogram construction - (a3d4249) - Volodia PAROL-GUARINO
- **(enoslib)** Nixify VM creation for G5k - (1b0fd23) - Volodia PAROL-GUARINO
- **(experiment)** Add new measures - (c3fd769) - Volodia PAROL-GUARINO
- **(experiment)** Add ration setting for low latency vs rest functions - (0fc259c) - Volodia PAROL-GUARINO
- **(experiment)** Remove waiting between trials - (d0f011c) - Volodia PAROL-GUARINO
- **(experiment)** Add print error message when request is rejected - (b40241a) - Volodia PAROL-GUARINO
- **(experiment)** Replace delay with arguments for max latency and number of functions, instead of having them tied together - (c462862) - Volodia PAROL-GUARINO
- **(experiment)** Add option to execute command while tunnels are opened, and close afterwards - (edb4aef) - Volodia PAROL-GUARINO
- **(experiment)** Label functions even when not provisioned - (22eb983) - Volodia PAROL-GUARINO
- **(experiment)** Make sure experiment deploys exactly n functions with random latencies - (1975c76) - Volodia PAROL-GUARINO
- **(experiment)** Start experiments (enoslib) in docker container generated through nix - (d1b79b4) - Volodia PAROL-GUARINO
- **(experiment)** Enable plots for ram and cpu + set ggplot global theme - (041fb39) - Volodia PAROL-GUARINO
- **(fog_node)** Add maxReplica - (7343501) - Volodia PAROL-GUARINO
- **(fog_node)** Limit both the function limits as well as the requested cpu and ram - (fbbff67) - Volodia PAROL-GUARINO
- **(fog_node)** Use defined cpu and memory instead of dynamically establising them - (be92420) - Volodia PAROL-GUARINO
- **(fog_node)** Add reserved memory & cpu to the RON configuration file + refactor file - (795c92b) - Volodia PAROL-GUARINO
- **(fog_node)** Add SLA seriaalized as JSON as an env var for the function deployed - (aa7a966) - Volodia PAROL-GUARINO
- **(fog_node)** Add edge_ward placement algorithm - (48044dc) - Volodia PAROL-GUARINO
- **(iot_emulation)** Add tracking for cron request failed - (2ac4900) - Volodia PAROL-GUARINO
- **(iot_emulation)** Add interval changing shortcut - (747c685) - Volodia PAROL-GUARINO
- **(result)** Use new settings for latencies - (cd5daab) - Volodia PAROL-GUARINO
- **(results)** Add heatmap for latencies - (785f247) - Volodia PAROL-GUARINO
- **(vm)** Upgrade deployment image and add tighter NTP sync options using chrony - (3573a47) - Volodia PAROL-GUARINO
- Move measuring logic from the iot_emulation to the function itself. - (2984286) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- **(experiment)** Remove useless Dockerfile - (5bd00c3) - Volodia PAROL-GUARINO
- **(iot_emulation)** Remove old file - (c86f481) - Volodia PAROL-GUARINO
- **(manager)** Enable CodeLLDB debugging - (136a026) - Volodia PAROL-GUARINO
- **(manager)** Fix clippy suggestions - (d24eea4) - Volodia PAROL-GUARINO
#### Refactoring
- **(experiment)** Update arguments and ways to pass them - (4701e65) - Volodia PAROL-GUARINO
- **(experiment)** Change plots to heatmaps + experience parameters - (69f767c) - Volodia PAROL-GUARINO
- **(fog_node)** Indicate rename function to reflect the set nature instead of update - (b49d400) - Volodia PAROL-GUARINO
- **(functions)** Update histogram buckets to use fixed margin - (5caf444) - Volodia PAROL-GUARINO
- **(manager)** Fix actix data type being create inside server's scope and not outside - (9e0e9fa) - Volodia PAROL-GUARINO
- **(vm)** Fix k3s deployment complaining with certificates and times - (167d362) - Volodia PAROL-GUARINO

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).