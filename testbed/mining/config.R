generate_gif <- FALSE
reload_big_data <- TRUE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
workers <- parallel::detectCores()
time_interval <- 15 # secs

no_memoization <- FALSE
single_graphs <- FALSE

GRAPH_ONE_COLUMN_HEIGHT <- 3
GRAPH_ONE_COLUMN_WIDTH <- 5
GRAPH_HALF_COLUMN_WIDTH <- 2.5
GRAPH_TWO_COLUMN_WIDTH <- 12

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  "metrics_valuation_rates.env_1a-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-26-14-47.tar.xz",
  "metrics_valuation_rates.env_1a-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-26-14-33.tar.xz",
  "metrics_valuation_rates.env_1a-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-03-26-15-02.tar.xz",
  "metrics_valuation_rates.env_1a-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-26-14-18.tar.xz",
  "metrics_valuation_rates.env_1b-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-27-19-36.tar.xz",
  "metrics_valuation_rates.env_1b-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-27-20-45.tar.xz",
  "metrics_valuation_rates.env_1b-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-27-20-10.tar.xz",
  "metrics_valuation_rates.env_1b-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-03-27-21-20.tar.xz",
  # --
  "metrics_valuation_rates.env_1c-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-28-14-16.tar.xz",
  "metrics_valuation_rates.env_1c-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-28-14-57.tar.xz",
  "metrics_valuation_rates.env_1c-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-28-13-39.tar.xz",
  # --
  "metrics_valuation_rates.env_1712047319-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-09-40.tar.xz",
  "metrics_valuation_rates.env_1712047319-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-09-13.tar.xz",
  "metrics_valuation_rates.env_1712047319-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-08-58.tar.xz",
  "metrics_valuation_rates.env_1712047319-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-02-09-27.tar.xz",
  "metrics_valuation_rates.env_1712059813-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-12-25.tar.xz",
  "metrics_valuation_rates.env_1712059813-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-12-41.tar.xz",
  "metrics_valuation_rates.env_1712059813-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-13-11.tar.xz",
  "metrics_valuation_rates.env_1712059813-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-02-12-56.tar.xz",
  #---
  #---
  #---
  "last element that is here because last element should not have any comma in the end and that sucks hard time."
)
METRICS_ARKS <- METRICS_ARKS[-length(METRICS_ARKS)]

library(stringr)

METRICS_GROUP <- str_match(METRICS_ARKS, "metrics_.*-fog_node_(.*)-market.*\\.tar\\.xz")
METRICS_GROUP <- METRICS_GROUP[, 2]
METRICS_GROUP_GROUP <- str_match(METRICS_ARKS, "metrics_.*\\.(.*)-fog_node_.*-market.*\\.tar\\.xz")
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
