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
  # "metrics_valuation_rates.env_1a-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-26-14-47.tar.xz",
  # "metrics_valuation_rates.env_1a-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-26-14-33.tar.xz",
  # "metrics_valuation_rates.env_1a-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-03-26-15-02.tar.xz",
  # "metrics_valuation_rates.env_1a-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-26-14-18.tar.xz",
  # "metrics_valuation_rates.env_1b-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-27-19-36.tar.xz",
  # "metrics_valuation_rates.env_1b-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-27-20-45.tar.xz",
  # "metrics_valuation_rates.env_1b-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-27-20-10.tar.xz",
  # "metrics_valuation_rates.env_1b-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-03-27-21-20.tar.xz",
  ## --
  # "metrics_valuation_rates.env_1c-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-28-14-16.tar.xz",
  # "metrics_valuation_rates.env_1c-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-28-14-57.tar.xz",
  # "metrics_valuation_rates.env_1c-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-03-28-13-39.tar.xz",
  ## --
  # "metrics_valuation_rates.env_1712047319-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-09-40.tar.xz",
  # "metrics_valuation_rates.env_1712047319-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-09-13.tar.xz",
  # "metrics_valuation_rates.env_1712047319-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-08-58.tar.xz",
  # "metrics_valuation_rates.env_1712047319-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-02-09-27.tar.xz",
  # "metrics_valuation_rates.env_1712059813-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-12-25.tar.xz",
  # "metrics_valuation_rates.env_1712059813-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-12-41.tar.xz",
  # "metrics_valuation_rates.env_1712059813-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-02-13-11.tar.xz",
  # "metrics_valuation_rates.env_1712059813-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-02-12-56.tar.xz",
  ## --
  # "metrics_valuation_rates.env_1712129480-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-07-42.tar.xz",
  # "metrics_valuation_rates.env_1712129480-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-07-58.tar.xz",
  # "metrics_valuation_rates.env_1712129480-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-07-49.tar.xz",
  # "metrics_valuation_rates.env_1712129480-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-03-08-06.tar.xz",
  ## --
  # "metrics_valuation_rates.env_11712152389-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-14-18.tar.xz",
  # "metrics_valuation_rates.env_11712152389-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-14-12.tar.xz",
  # "metrics_valuation_rates.env_11712152389-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-14-03.tar.xz",
  # "metrics_valuation_rates.env_11712152389-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-03-14-26.tar.xz",
  # "metrics_valuation_rates.env_1712142651-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-11-36.tar.xz",
  # "metrics_valuation_rates.env_1712142651-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-11-29.tar.xz",
  # "metrics_valuation_rates.env_1712142651-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-03-11-20.tar.xz",
  # "metrics_valuation_rates.env_21712152389-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-14-26.tar.xz",
  # "metrics_valuation_rates.env_21712152389-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-14-12.tar.xz",
  # "metrics_valuation_rates.env_21712152389-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-03-14-20.tar.xz",
  # "metrics_valuation_rates.env_21712152389-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-03-14-04.tar.xz",
  ## --
  # "metrics_valuation_rates.env_11712214235-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-07-23.tar.xz",
  # "metrics_valuation_rates.env_11712214235-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-07-37.tar.xz",
  # "metrics_valuation_rates.env_11712214235-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-07-16.tar.xz",
  # "metrics_valuation_rates.env_11712214235-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-04-07-30.tar.xz",
  # "metrics_valuation_rates.env_21712216345-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-07-54.tar.xz",
  # "metrics_valuation_rates.env_21712216345-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-08-10.tar.xz",
  # "metrics_valuation_rates.env_21712216345-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-04-08-02.tar.xz",
  ## --
  "metrics_valuation_rates.env_11712220335-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-09-21.tar.xz",
  "metrics_valuation_rates.env_11712220335-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-09-06.tar.xz",
  "metrics_valuation_rates.env_11712220335-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-09-15.tar.xz",
  "metrics_valuation_rates.env_21712222587-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-09-49.tar.xz",
  "metrics_valuation_rates.env_21712222587-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-09-34.tar.xz",
  "metrics_valuation_rates.env_21712222587-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-04-09-56.tar.xz",
  "metrics_valuation_rates.env_21712222587-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-04-09-42.tar.xz",
  ## --
  "metrics_valuation_rates.env_11712304036-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-05-08-41.tar.xz",
  "metrics_valuation_rates.env_11712304036-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-05-08-24.tar.xz",
  "metrics_valuation_rates.env_11712304036-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-05-08-16.tar.xz",
  "metrics_valuation_rates.env_11712304036-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-05-08-33.tar.xz",
  #--
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
options(width = 1000)

# Not scientific notations
options(scipen = 10000)
options(width = 1000)

# Not scientific notations
options(scipen = 10000)
options(width = 1000)

# Not scientific notations
options(scipen = 10000)
options(scipen = 10000)
options(scipen = 10000)
options(scipen = 10000)
options(scipen = 10000)
options(scipen = 10000)
options(scipen = 10000)
