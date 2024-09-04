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
  # "metrics_valuation_rates.env_1_1723213841-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-08-09-15-15.tar.xz",
  # "metrics_valuation_rates.env_1_1723213841-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-08-09-15-45.tar.xz",
  # "metrics_valuation_rates.env_1_1723213841-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-08-09-14-57.tar.xz",
  # "metrics_valuation_rates.env_1_1723450373-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-08-12-08-44.tar.xz",
  # "metrics_valuation_rates.env_1_1723450373-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-08-12-09-39.tar.xz",
  # "metrics_valuation_rates.env_1_1723450373-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-08-12-09-19.tar.xz",
  # "metrics_valuation_rates.env_1_1723473135-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-08-12-16-02.tar.xz",
  # "metrics_valuation_rates.env_1_1723473135-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-08-12-14-58.tar.xz",
  # "metrics_valuation_rates.env_1_1723473135-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-08-12-15-41.tar.xz",
  # "metrics_valuation_rates.env_1_12-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-08-13-11-47.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-09-02-09-18.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-09-02-09-47.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-09-02-10-14.tar.xz",
  # "metrics_valuation_rates.env_1_1725273464-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-02-10-54.tar.xz",
  # "metrics_valuation_rates.env_1_1725276024-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-02-11-35.tar.xz",
  # "metrics_valuation_rates.env_1_1725278732-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-02-12-45.tar.xz",
  ## ---
  "metrics_valuation_rates.env_1_1725284786-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-02-14-38.tar.xz",
  "metrics_valuation_rates.env_1_1725284786-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-02-14-07.tar.xz",
  "metrics_valuation_rates.env_1_1725284786-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-02-14-23.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-09-03-07-46.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-09-03-09-02.tar.xz",
  "metrics_valuation_rates.env_1_1725355182-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-03-09-38.tar.xz",
  "metrics_valuation_rates.env_1_1725355182-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-03-10-09.tar.xz",
  "metrics_valuation_rates.env_1_1725355182-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-03-09-53.tar.xz",
  "metrics_valuation_rates.env_1_1725361412-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-03-11-26.tar.xz",
  "metrics_valuation_rates.env_1_1725361412-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-03-11-57.tar.xz",
  "metrics_valuation_rates.env_1_1725361412-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-03-11-41.tar.xz",
  "metrics_valuation_rates.env_1_1725365160-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-03-12-55.tar.xz",
  "metrics_valuation_rates.env_1_1725365160-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-03-12-39.tar.xz",
  "metrics_valuation_rates.env_1_1725365160-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-03-12-23.tar.xz",
  "metrics_valuation_rates.env_1_1725371375-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-03-15-12.tar.xz",
  "metrics_valuation_rates.env_1_1725371375-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-03-14-45.tar.xz",
  "metrics_valuation_rates.env_1_1725371375-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-03-14-18.tar.xz",
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
