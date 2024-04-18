generate_gif <- FALSE
reload_big_data <- TRUE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
all_workers <- parallel::detectCores()
workers <- min(all_workers, 8)
time_interval <- 15 # secs

no_memoization <- FALSE
single_graphs <- FALSE

GRAPH_ONE_COLUMN_HEIGHT <- 3
GRAPH_ONE_COLUMN_WIDTH <- 5
GRAPH_HALF_COLUMN_WIDTH <- 2.5
GRAPH_TWO_COLUMN_WIDTH <- 12

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  "metrics_valuation_rates.env_11_1713292103_.env.3-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-19-16.tar.xz",
  "metrics_valuation_rates.env_11_1713292103_.env.3-fog_node-edge_first-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-18-43.tar.xz",
  "metrics_valuation_rates.env_11_1713292103_.env.3-fog_node-edge_furthest-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-19-06.tar.xz",
  # "metrics_valuation_rates.env_11_1713292103_.env.3-fog_node-maxcpu-cpu_ratio_rates-no_telemetry-market-random-no_telemetry_2024-04-16-18-55.tar.xz",
  "metrics_valuation_rates.env_1_1713281750_.env.2-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-16-21.tar.xz",
  "metrics_valuation_rates.env_1_1713281750_.env.2-fog_node-edge_first-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-16-11.tar.xz",
  "metrics_valuation_rates.env_1_1713281750_.env.2-fog_node-edge_furthest-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-15-50.tar.xz",
  # "metrics_valuation_rates.env_1_1713281750_.env.2-fog_node-maxcpu-cpu_ratio_rates-no_telemetry-market-random-no_telemetry_2024-04-16-16-01.tar.xz",
  "metrics_valuation_rates.env_2_1713281750_.env.1-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-15-49.tar.xz",
  "metrics_valuation_rates.env_2_1713281750_.env.1-fog_node-edge_first-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-16-01.tar.xz",
  "metrics_valuation_rates.env_2_1713281750_.env.1-fog_node-edge_furthest-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-16-11.tar.xz",
  # "metrics_valuation_rates.env_2_1713281750_.env.1-fog_node-maxcpu-cpu_ratio_rates-no_telemetry-market-random-no_telemetry_2024-04-16-16-21.tar.xz",
  "metrics_valuation_rates.env_4_1713284555_.env.1-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-16-52.tar.xz",
  "metrics_valuation_rates.env_4_1713284555_.env.1-fog_node-edge_first-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-16-39.tar.xz",
  "metrics_valuation_rates.env_4_1713284555_.env.1-fog_node-edge_furthest-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-17-17.tar.xz",
  # "metrics_valuation_rates.env_4_1713284555_.env.1-fog_node-maxcpu-cpu_ratio_rates-no_telemetry-market-random-no_telemetry_2024-04-16-17-05.tar.xz",
  "metrics_valuation_rates.env_5_1713284593_.env.1-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-17-23.tar.xz",
  "metrics_valuation_rates.env_5_1713284593_.env.1-fog_node-edge_furthest-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-17-08.tar.xz",
  # "metrics_valuation_rates.env_5_1713284593_.env.1-fog_node-maxcpu-cpu_ratio_rates-no_telemetry-market-random-no_telemetry_2024-04-16-16-54.tar.xz",
  "metrics_valuation_rates.env_7_1713287954_.env.3-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-18-07.tar.xz",
  "metrics_valuation_rates.env_7_1713287954_.env.3-fog_node-edge_first-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-17-54.tar.xz",
  # "metrics_valuation_rates.env_7_1713287954_.env.3-fog_node-maxcpu-cpu_ratio_rates-no_telemetry-market-random-no_telemetry_2024-04-16-18-20.tar.xz",
  "metrics_valuation_rates.env_8_1713288368_.env.2-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-18-11.tar.xz",
  "metrics_valuation_rates.env_8_1713288368_.env.2-fog_node-edge_furthest-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-18-26.tar.xz",
  "metrics_valuation_rates.env_9_1713288983_.env.1-fog_node-auction-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-18-22.tar.xz",
  "metrics_valuation_rates.env_9_1713288983_.env.1-fog_node-edge_first-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-18-08.tar.xz",
  "metrics_valuation_rates.env_9_1713288983_.env.1-fog_node-edge_furthest-quadratic_rates-no_telemetry-market-default_strategy-no_telemetry_2024-04-16-18-37.tar.xz",
  #---
  #---
  #---
  "last element that is here because last element should not have any comma in the end and that sucks hard time."
)
METRICS_ARKS <- METRICS_ARKS[-length(METRICS_ARKS)]

library(stringr)

METRICS_GROUP <- str_match(METRICS_ARKS, "metrics_.*-fog_node-(.*)-market.*\\.tar\\.xz")
METRICS_GROUP <- METRICS_GROUP[, 2]
METRICS_GROUP_GROUP <- str_match(METRICS_ARKS, "metrics_.*\\.(.*)-fog_node-.*-market.*\\.tar\\.xz")
METRICS_GROUP_GROUP <- METRICS_GROUP_GROUP[, 2]

length(METRICS_ARKS)
length(METRICS_GROUP)
length(METRICS_GROUP_GROUP)
stopifnot(length(METRICS_ARKS) == length(METRICS_GROUP))
stopifnot(length(METRICS_ARKS) == length(METRICS_GROUP_GROUP))

# Make the output of the console real wide
# alt+z on the vscode console to make it not wrap
options(width = 1000)

# Not scientific notations
options(scipen = 10000)
