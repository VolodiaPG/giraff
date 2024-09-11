generate_gif <- FALSE
reload_big_data <- TRUE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
all_workers <- parallel::detectCores()
workers <- min(all_workers, 6)
time_interval <- 15 # secs

no_memoization <- FALSE
single_graphs <- TRUE

GRAPH_ONE_COLUMN_HEIGHT <- 3
GRAPH_ONE_COLUMN_WIDTH <- 5
GRAPH_HALF_COLUMN_WIDTH <- 2.5
GRAPH_TWO_COLUMN_WIDTH <- 12

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  # "metrics_valuation_rates.env_DEV-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.dev_2024-09-06-09-27.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.dev_2024-09-06-09-51.tar.xz",
  # "metrics_valuation_rates.env_1_1725616570-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-06-10-24.tar.xz",
  # "metrics_valuation_rates.env_1_1725616570-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-06-10-40.tar.xz",
  # "metrics_valuation_rates.env_1_1725624801-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-06-12-29.tar.xz",
  # "metrics_valuation_rates.env_1_1725867960-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-09-08-20.tar.xz",
  # "metrics_valuation_rates.env_1_1725867960-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-09-08-37.tar.xz",
  # "metrics_valuation_rates.env_1_1725867960-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-09-08-04.tar.xz",
  # "metrics_valuation_rates.env_1_1725873606-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-09-09-36.tar.xz",
  # "metrics_valuation_rates.env_1_1725879756-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-09-11-48.tar.xz",
  # "metrics_valuation_rates.env_1_1725879756-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-09-11-26.tar.xz",
  # "metrics_valuation_rates.env_1_1725879756-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-09-12-10.tar.xz",
  # "metrics_valuation_rates.env_1_1725884693-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-09-13-05.tar.xz",
  # "metrics_valuation_rates.env_1_1725884693-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-09-13-24.tar.xz",
  # "metrics_valuation_rates.env_1_1725884693-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-09-12-46.tar.xz",
  # "metrics_valuation_rates.env_1_1725959049-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-09-28.tar.xz",
  # "metrics_valuation_rates.env_1_1725959049-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-09-20.tar.xz",
  # "metrics_valuation_rates.env_1_1725959049-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-10-09-12.tar.xz",
  # "metrics_valuation_rates.env_1_1725962498-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-10-27.tar.xz",
  # "metrics_valuation_rates.env_1_1725962498-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-10-20.tar.xz",
  # "metrics_valuation_rates.env_1_1725962498-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-10-10-10.tar.xz",
  # "metrics_valuation_rates.env_1_1725969445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-12-04.tar.xz",
  # "metrics_valuation_rates.env_1_1725969445-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-12-11.tar.xz",
  # "metrics_valuation_rates.env_1_1725969445-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-10-12-18.tar.xz",
  "metrics_valuation_rates.env_1_1725978602-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-14-45.tar.xz",
  "metrics_valuation_rates.env_1_1725978602-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-10-14-51.tar.xz",
  "metrics_valuation_rates.env_1_1725978602-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-10-14-37.tar.xz",
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
