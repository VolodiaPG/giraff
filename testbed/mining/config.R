generate_gif <- FALSE
reload_big_data <- TRUE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
workers <- parallel::detectCores()
time_interval <- 15 # secs

no_memoization <- FALSE
single_graphs <- TRUE

GRAPH_ONE_COLUMN_HEIGHT <- 3
GRAPH_ONE_COLUMN_WIDTH <- 5
GRAPH_HALF_COLUMN_WIDTH <- 2.5
GRAPH_TWO_COLUMN_WIDTH <- 12

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  # "metrics_valuation_rates.env_11712567784-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-09-52.tar.xz",
  # "metrics_valuation_rates.env_11712567784-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-09-27.tar.xz",
  # "metrics_valuation_rates.env_11712567784-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-09-36.tar.xz",
  # "metrics_valuation_rates.env_11712567784-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-09-45.tar.xz",
  # "metrics_valuation_rates.env_11712574656-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-11-19.tar.xz",
  # "metrics_valuation_rates.env_11712574656-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-11-26.tar.xz",
  # "metrics_valuation_rates.env_11712574656-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-11-33.tar.xz",
  # "metrics_valuation_rates.env_11712574656-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-11-40.tar.xz",
  # "metrics_valuation_rates.env_21712570034-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-10-09.tar.xz",
  # "metrics_valuation_rates.env_21712576516-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-11-54.tar.xz",
  # "metrics_valuation_rates.env_21712576516-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-12-11.tar.xz",
  # "metrics_valuation_rates.env_21712576516-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-12-03.tar.xz",
  #---
  # "metrics_valuation_rates.env_11712580463-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-13-24.tar.xz",
  # "metrics_valuation_rates.env_11712580463-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-13-03.tar.xz",
  # "metrics_valuation_rates.env_11712580463-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-13-34.tar.xz",
  # "metrics_valuation_rates.env_11712580463-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-13-13.tar.xz",
  # "metrics_valuation_rates.env_21712583331-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-14-01.tar.xz",
  # "metrics_valuation_rates.env_21712583331-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-14-11.tar.xz",
  # "metrics_valuation_rates.env_21712583331-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-14-21.tar.xz",
  ## "metrics_valuation_rates.env_21712583331-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-13-51.tar.xz",
  "metrics_valuation_rates.env_11712601354-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-19-30.tar.xz",
  "metrics_valuation_rates.env_11712601354-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-19-53.tar.xz",
  "metrics_valuation_rates.env_11712601354-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-20-17.tar.xz",
  "metrics_valuation_rates.env_11712601354-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-19-05.tar.xz",
  "metrics_valuation_rates.env_21712601354-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-20-19.tar.xz",
  "metrics_valuation_rates.env_21712601354-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-19-05.tar.xz",
  "metrics_valuation_rates.env_21712601354-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-19-55.tar.xz",
  "metrics_valuation_rates.env_21712601354-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-19-30.tar.xz",
  "metrics_valuation_rates.env_51712601360-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-19-54.tar.xz",
  "metrics_valuation_rates.env_51712601360-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-19-06.tar.xz",
  "metrics_valuation_rates.env_51712601360-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-20-18.tar.xz",
  "metrics_valuation_rates.env_51712601360-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-19-30.tar.xz",
  "metrics_valuation_rates.env_61712607525-fog_node_auction_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-20-49.tar.xz",
  "metrics_valuation_rates.env_61712607525-fog_node_edge_first_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-22-03.tar.xz",
  "metrics_valuation_rates.env_61712607525-fog_node_edge_furthest_quadratic_rates_no-telemetry-market_default-strategy_no-telemetry_2024-04-08-21-14.tar.xz",
  "metrics_valuation_rates.env_61712607525-fog_node_powerrandom_powerrandom_rates_no-telemetry-market_powerrandom_no-telemetry_2024-04-08-21-39.tar.xz",
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
