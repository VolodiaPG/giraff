- - -
## 2.0.0 - 2023-10-11
#### Bug Fixes
- **(testbed)** Reversed nixpkgs to previous working version to make the testbed run again - (94c4c4b) - Volodia PAROL-GUARINO
- make changelog not bugged - (1a5afeb) - Volodia PAROL-GUARINO
- fix update that broke stuff (reverted) - (1a2ec29) - Volodia PAROL-GUARINO
- fix paths not updated after refactor - (2a6c005) - Volodia PAROL-GUARINO
#### Features
- use kubenix for reproducible deployment of openfaas - (dc4f2e9) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- **(update)** update lock - (4d7403c) - Volodia PAROL-GUARINO
- **(update)** fix poetry2nix having issues with nixpkgs - (1c03742) - Volodia PAROL-GUARINO
- **(update)** update lock - (1f93b89) - Volodia PAROL-GUARINO
- remove already ignored dir - (fe9925b) - Volodia PAROL-GUARINO
- fix missing paths for devshells - (c613c15) - Volodia PAROL-GUARINO
- remove unused vpn options - (1058971) - Volodia PAROL-GUARINO
#### Refactoring
- Unify toolchains - (555065e) - Volodia PAROL-GUARINO
- Unify flakes - (034ac3b) - Volodia PAROL-GUARINO
- move enos code into testbed dir - (baced4a) - Volodia PAROL-GUARINO

- - -

## 1.0.0 - 2023-10-04
#### Bug Fixes
- **(echo-function)** better to sort the buckets before remove duplicates, as it is required per the documentation - (4e54f34) - Volodia PAROL-GUARINO
- **(experiment)** use memsuspend instead of memfree + fix missing SIZE_MULTIPLIER env var - (a0ff410) - Volodia PAROL-GUARINO
- **(experiment)** Fix networking between iot_emulation and proxies - (f7a9851) - Volodia PAROL-GUARINO
- **(experiment)** Move copy of env and definition files before parralelizing them - (fbf66e1) - Volodia PAROL-GUARINO
- **(experiment)** Fix enoslib version to use SMT by default + small fixes - (bbebcc7) - Volodia PAROL-GUARINO
- **(experiment)** Allow for specification of a different local port - (54a7912) - Volodia PAROL-GUARINO
- **(experiment)** Add sensible default env variables to run outside a container - (1825544) - Volodia PAROL-GUARINO
- **(experiment)** Fix envrc - (7fec46d) - Volodia PAROL-GUARINO
- **(fog_node)** Fix pricing ratio not set correctly - (8738421) - Volodia PAROL-GUARINO
- **(fog_node)** Remove function mem+cpu utilization from the metrics instead of adding them, again, when unprovisioning a function - (3a4646f) - Volodia PAROL-GUARINO
- **(fog_node)** Add function tracking repository in the dynamic dispatch of actix to be able to actually measure something - (6304218) - Volodia PAROL-GUARINO
- **(functions)** Fix unmatching datatypes in dynamic dispatch - (b625bed) - Volodia PAROL-GUARINO
- **(functions)** Go back to only using an histogram - (97ea26f) - Volodia PAROL-GUARINO
- **(functions)** Fix histograms throwning errors because vector was not an ordered set - (fe478f2) - Volodia PAROL-GUARINO
- **(functions)** Fix micondifurations in flake - (3c648be) - Volodia PAROL-GUARINO
- **(iot_emulation)** Missing loading of nix-generated container image that was not updated on any remote because of this - (a29ace4) - Volodia PAROL-GUARINO
- **(manager)** Fix DOCKER env variable not being settable by the user - (4a8b8ec) - Volodia PAROL-GUARINO
- **(manager)** Fix nix develshell - (aeb83b3) - Volodia PAROL-GUARINO
- **(manager)** Add removal of used resources when functions gets removed - (eb152c1) - Volodia PAROL-GUARINO
- **(manager)** Fix generation for docker images - (92bb9ec) - Volodia PAROL-GUARINO
- **(pre-commit)** Fix wrong path passed to script - (a83caf8) - Volodia PAROL-GUARINO
- **(simulator)** fix missing level not being displayed - (65c6ed5) - Volodia PAROL-GUARINO
- **(simulator)** fix incorrect use of sorted that doesn't sort in place - (f61267b) - Volodia PAROL-GUARINO
- **(simulator)** fix price monitoring not working - (fa64bc6) - Volodia PAROL-GUARINO
- remove junk files - (e4ab548) - Volodia PAROL-GUARINO
- Add csv and bak files to the gitignore - (2bcd97c) - Volodia PAROL-GUARINO
#### Documentation
- **(experiment)** add legends for graphs - (32d5fd2) - Volodia PAROL-GUARINO
- add link to released raw data - (cce7d7d) - Volodia PAROL-GUARINO
- Change h2 to h3 - (facc996) - Volodia PAROL-GUARINO
- add mascot - (e4a0e88) - Volodia PAROL-GUARINO
- update ToC - (3729d1a) - Volodia PAROL-GUARINO
- update readme - (6c67401) - Volodia PAROL-GUARINO
#### Features
- **(README)** Update - (d392397) - Volodia PAROL-GUARINO
- **(echo-function)** Add debug entry to test locally - (8a83aba) - Volodia PAROL-GUARINO
- **(experience)** Add .env file support + load settings support - (03880e0) - Volodia PAROL-GUARINO
- **(experiment)** Add working experiment deployment - (57ed654) - Volodia PAROL-GUARINO
- **(experiment)** run experiments from inside g5k to fix the shitstorm running outside actually is... - (c5bae44) - Volodia PAROL-GUARINO
- **(experiment)** add VM generation for running experiments from g5k - (ef7c4af) - Volodia PAROL-GUARINO
- **(experiment)** add dry experiemnt to measure the number of used nodes without deploying - (f90d7e0) - Volodia PAROL-GUARINO
- **(experiment)** add support for randomly generated networks in realworld deployments - (05ca502) - Volodia PAROL-GUARINO
- **(experiment)** add retries and timeout for experiments - (b6c583e) - Volodia PAROL-GUARINO
- **(experiment)** Add support for parallel and configurable experiments - (e561bf9) - Volodia PAROL-GUARINO
- **(experiment)** add randomizer for size of virtual machines - (4df6b39) - Volodia PAROL-GUARINO
- **(experiment)** better experiments with network size variation and random generation - (0bd069e) - Volodia PAROL-GUARINO
- **(experiment)** Add multidplyr - (de636fc) - Volodia PAROL-GUARINO
- **(experiment)** Add animation to visualize - (50def0b) - Volodia PAROL-GUARINO
- **(experiment)** Make the VM stateless - (e30b672) - Volodia PAROL-GUARINO
- **(experiment)** Add graph utilizing the repartition of low latency functions - (b030807) - Volodia PAROL-GUARINO
- **(experiment)** Utilize sla_id - (6cd99ec) - Volodia PAROL-GUARINO
- **(experiment)** Enable large-sclae experiments - (d0f410e) - Volodia PAROL-GUARINO
- **(experiment)** Add function sizes + archive exploitation in R script - (7c3e5c9) - Volodia PAROL-GUARINO
- **(experiment)** Lock versions of openfaas-related containers in place - (4f078f4) - Volodia PAROL-GUARINO
- **(experiment)** Parallelize deployment on g5k - (5e74eb3) - Volodia PAROL-GUARINO
- **(experiment)** Add function type in tag - (d849291) - Volodia PAROL-GUARINO
- **(experiment)** Migrate to nix-declared jupyterlab - (0b4ae0e) - Volodia PAROL-GUARINO
- **(experiment)** Allow for independant scenario configuration from the local machine's - (4177ca9) - Volodia PAROL-GUARINO
- **(experiment)** Add data collecting in parrallel an much more space optimized - (11eadec) - Volodia PAROL-GUARINO
- **(experiments)** Add macos support - (fde8ca0) - Volodia P.-G
- **(experiments)** Add common requests between runs in the same container - (8c7c355) - Volodia PAROL-GUARINO
- **(experiments)** Add pickeling of function request configurations - (8cff9ff) - Volodia PAROL-GUARINO
- **(fog_node)** Add sla id to bid gauge and add new metric SLA_SEEN to track down the path of a unique SLA - (38ddb03) - Volodia PAROL-GUARINO
- **(fog_node)** Add valuation using rates - (dadb249) - Volodia PAROL-GUARINO
- **(functions)** Add sla id to the prom tags - (01e56b5) - Volodia PAROL-GUARINO
- **(functions)** Add accurate measurements points for data - (f7716bc) - Volodia PAROL-GUARINO
- **(iot_emulation)** Build with docker layered - (43164cc) - Volodia PAROL-GUARINO
- **(iot_emulation)** Remove functions if they are not found on openfaas - (f43f974) - Volodia PAROL-GUARINO
- **(iso)** Make sure to preempt like on a server - (664c8ef) - Volodia PAROL-GUARINO
- **(manager)** Export prometheus metrics as stream using a pool of buffers to ease on frequent allocations - (04686c6) - Volodia PAROL-GUARINO
- **(manager)** Use exp moving average for latencies - (8b192ce) - Volodia PAROL-GUARINO
- **(manager)** Add new models + Id field to the SLA - (4f7bc30) - Volodia PAROL-GUARINO
- **(manager)** Add sla ID - (cd8e16a) - Volodia PAROL-GUARINO
- **(manager)** Add provisioned and refused function counters - (052f0fd) - Volodia PAROL-GUARINO
- **(manager)** Add local running without debug as a new option - (49aede7) - Volodia PAROL-GUARINO
- **(paper)** experimental config - (dd3438a) - Volodia PAROL-GUARINO
- **(simulator)** add nb of functions as parameters and allow for random experimental settings (expe.py) regeneration in chunks - (3fff5cf) - Volodia PAROL-GUARINO
- **(simulator)** add more precisie bid metrics and avoid duplication of metrics - (83807aa) - Volodia PAROL-GUARINO
- **(simulator)** add logistic pricing function - (ab034a8) - Volodia PAROL-GUARINO
- **(simulator)** add result importing into the jupyter notebook - (e7f907e) - Volodia PAROL-GUARINO
- **(simulator)** add random flavor and result export to csv - (a8f2ead) - Volodia PAROL-GUARINO
- **(simulator)** add furthest placement methods (look for furthest node in the tree) - (56a6561) - Volodia PAROL-GUARINO
- **(simulator)** add result analysis at then end - (116557f) - Volodia PAROL-GUARINO
- **(simulator)** add linear princing per part - (8890eb6) - Volodia PAROL-GUARINO
- **(simulator)** Add parallel for running experiments - (3c6296c) - Volodia PAROL-GUARINO
- **(simulator)** Add monitoring to earnings - (85797d5) - Volodia PAROL-GUARINO
- **(simulator)** Add linear cost model - (d11925d) - Volodia PAROL-GUARINO
- **(simulator)** Add visitor pattern for cost estimation and set seed for all random involved - (272c906) - Volodia PAROL-GUARINO
- **(simulator)** Enable simulation using configuration defined for experimentation - (e8b1d87) - Volodia PAROL-GUARINO
- **(simulator)** Init the simulator - (3805728) - Volodia PAROL-GUARINO
- **(stat)** Add back metrics used for the paper - (571ebd7) - Volodia PAROL-GUARINO
- Add redirection of logs to files when running an experiment - (497b346) - Volodia PAROL-GUARINO
- Working large scale deployments - (8702e45) - Volodia PAROL-GUARINO
- workable graphs - (a6e6813) - Volodia PAROL-GUARINO
- Add InfluxDB2 push model - (b81d608) - Volodia PAROL-GUARINO
- small updates and optimizations - (245d2cd) - Volodia PAROL-GUARINO
- Add moving median and uncertainties - (49f3fc2) - Volodia PAROL-GUARINO
- Use ICMP for latency estimation - (6185a1a) - Volodia PAROL-GUARINO
- paper ready - (3003d5d) - Volodia PAROL-GUARINO
- Add load definition per function entry - (4603193) - Volodia PAROL-GUARINO
- Add envfile for experiments - (b7c821d) - Volodia PAROL-GUARINO
- Add reservation duration to sla - (02218d2) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- **(experiment)** Lower expectations for experiments - (414714c) - Volodia P.-G
- **(experiment)** Update price - (6a729ab) - Volodia P.-G
- **(experiment)** Update config - (8ab7299) - Volodia P.-G
- **(experiment)** mypy fixes - (f647bf0) - Volodia PAROL-GUARINO
- **(experiment)** change experimental settings - (4b87fc0) - Volodia PAROL-GUARINO
- **(experiment)** remove mistakenly created symlink - (b973b35) - Volodia PAROL-GUARINO
- **(experiment)** remove old script - (a3d3ef4) - Volodia PAROL-GUARINO
- **(experiment)** remove comments - (5a7bd60) - Volodia PAROL-GUARINO
- **(experiment)** Update settings - (0fabd36) - Volodia PAROL-GUARINO
- **(experiment)** Update dependencies - (f73db0c) - Volodia PAROL-GUARINO
- **(experiment)** Add reservation duration as dotenv parameter - (de37000) - Volodia PAROL-GUARINO
- **(fmt)** Fix fmt - (17ad5b4) - Volodia PAROL-GUARINO
- **(functions)** Update of-watchdog to 0.9.11 - (b644ebc) - Volodia PAROL-GUARINO
- **(manager)** Add missing port to debug prometheus configuration - (edc2562) - Volodia PAROL-GUARINO
- **(manager)** Update deps - (2f26ae1) - Volodia PAROL-GUARINO
- **(manager)** Use more idiomatic code - (e1595d3) - Volodia PAROL-GUARINO
- **(manager)** Change justfile to ignore clippy when developing but only use it when pre commiting - (41ccb84) - Volodia PAROL-GUARINO
- **(paper)** add missing metrics in the gitignore - (f073bac) - Volodia PAROL-GUARINO
- **(stat)** change name from auctions to GIRAFF - (b94aa45) - Volodia PAROL-GUARINO
- **(update)** update nix deps for manager and iot_emulation - (2f77c39) - Volodia PAROL-GUARINO
- **(version)** 1.0.0 - (9aa421f) - Volodia PAROL-GUARINO
- update - (39f749d) - Volodia PAROL-GUARINO
- Update gitattributes to ignore Cargo.nix files - (6bb03e9) - Volodia PAROL-GUARINO
#### Refactoring
- **(experiment)** Polish the experiments - (3f71bcf) - Volodia PAROL-GUARINO
- **(experiment)** Utilize enos modification to open local port configured by the user - (11f034a) - Volodia PAROL-GUARINO
- **(experiment)** Change the experimental scenario - (b81389f) - Volodia PAROL-GUARINO
- **(experiments)** fix checks - (39f5d1e) - Volodia P.-G
- **(fog_node)** Use cron repository instead of small infinite loop - (6498b74) - Volodia PAROL-GUARINO
- **(iso)** rename squid to proxy - (c6988b4) - Volodia PAROL-GUARINO
- **(iso)** make pkgs not redefine all outputs - (e8425c8) - Volodia PAROL-GUARINO
- **(iso)** move common deps into base.nix - (d05e125) - Volodia PAROL-GUARINO
- **(iso)** add argument to pass modules - (9ef9812) - Volodia PAROL-GUARINO
- **(manager)** Wrap reqwest deserializer for better error retransmission and display of actual message that caused the error - (45a51a0) - Volodia PAROL-GUARINO
- **(manager)** Abandon thiserror and migrate to anyhow for context abilities - (bd8ee34) - Volodia PAROL-GUARINO
- **(manager)** Remove useless fields in sla and add reservation end - (5a1024d) - Volodia PAROL-GUARINO
- **(simulator)** allow for profiles with random base price - (ab29273) - Volodia PAROL-GUARINO
- **(simulator)** change the experiment settings to be more intuitive and add graphs - (bded93c) - Volodia PAROL-GUARINO
- **(simulator)** compute max level of fog net instead of using hard coded values - (c880249) - Volodia PAROL-GUARINO
- use upstreal alejandra from nixpkgs instead of flake - (9f5665b) - Volodia P.-G
- Correct flakes with higher standard - (ab0b6cf) - Volodia PAROL-GUARINO
- Allow SSH configuration + update experiment orders and configuration to take advantage of the stateless VMs - (e0a1bcb) - Volodia PAROL-GUARINO
- Remove useless traits and do massive overhaul - (b2ff578) - Volodia PAROL-GUARINO

- - -

## 0.10.0 - 2023-10-04
#### Features
- Add support for pre commits checks - (2d1634c) - Volodia PAROL-GUARINO
- Use nix as devenv - (ad2f558) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- **(functions)** Update cargo lock - (7fc0111) - Volodia PAROL-GUARINO
- **(gitignore)** Ignore pre commit config files - (c521d91) - Volodia PAROL-GUARINO
- **(manager)** Remove unused dependencies - (414a656) - Volodia PAROL-GUARINO
- **(version)** 0.10.0 - (c652c0e) - Volodia PAROL-GUARINO
- enable statix for pre-commit - (c59041e) - Volodia PAROL-GUARINO
- Replace formatter by alejandra and remove statix - (288c318) - Volodia PAROL-GUARINO
- Remove devcontainers - (ec9a0ad) - Volodia PAROL-GUARINO

- - -

## 0.9.0 - 2023-10-04
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
- **(version)** 0.9.0 - (a929342) - Volodia PAROL-GUARINO
#### Refactoring
- **(experiment)** Update arguments and ways to pass them - (4701e65) - Volodia PAROL-GUARINO
- **(experiment)** Change plots to heatmaps + experience parameters - (69f767c) - Volodia PAROL-GUARINO
- **(fog_node)** Indicate rename function to reflect the set nature instead of update - (b49d400) - Volodia PAROL-GUARINO
- **(functions)** Update histogram buckets to use fixed margin - (5caf444) - Volodia PAROL-GUARINO
- **(manager)** Fix actix data type being create inside server's scope and not outside - (9e0e9fa) - Volodia PAROL-GUARINO
- **(vm)** Fix k3s deployment complaining with certificates and times - (167d362) - Volodia PAROL-GUARINO

- - -

## 0.8.0 - 2023-10-04
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
- **(version)** 0.8.0 - (dc073f2) - Volodia PAROL-GUARINO
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

## 0.7.0 - 2023-10-04
#### Bug Fixes
- **(git)** Hide ark files - (92c2a09) - Volodia PAROL-GUARINO
#### Features
- **(fog_node)** Change healt hroute pings from HEAD to GET requests - (35892a2) - Volodia PAROL-GUARINO
- **(g5k)** Add network and htop monitors and increase iot_emulation resources - (3d59c51) - Volodia PAROL-GUARINO
- **(iot_emulation)** Add jaeger flag - (d842a52) - Volodia PAROL-GUARINO
- **(manager)** Enable full LTO & more optimizations - (8149b06) - Volodia PAROL-GUARINO
- **(results)** Add SLA violation graphs - (60350c0) - Volodia PAROL-GUARINO
#### Miscellaneous Chores
- **(version)** 0.7.0 - (92f46cb) - Volodia PAROL-GUARINO
- Remove deprecated and old stuff - (43f4da8) - Volodia PAROL-GUARINO
#### Refactoring
- **(experiment)** Change parameters - (3490265) - Volodia PAROL-GUARINO

- - -

## 0.6.0 - 2023-10-04

- - -

## 0.5.1 - 2023-10-04

- - -

## 0.5.0 - 2023-10-04

- - -

## 0.4.0 - 2023-10-04

- - -

## 0.3.0 - 2023-10-04

- - -

## 0.2.2 - 2023-10-04

- - -

## 0.2.1 - 2023-10-04

- - -

## 0.2.0 - 2023-10-04

- - -

## 0.1.0 - 2023-10-04

- - -

## 0.0.1 - 2023-10-04

