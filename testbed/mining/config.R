generate_gif <- FALSE
reload_big_data <- TRUE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
all_workers <- parallel::detectCores()
workers <- min(all_workers, 6)
time_interval <- 15 # secs

no_memoization <- FALSE
single_graphs <- FALSE

CHAIN_LENGTH <- 3

GRAPH_ONE_COLUMN_HEIGHT <- 3
GRAPH_ONE_COLUMN_WIDTH <- 3.6
GRAPH_HALF_COLUMN_WIDTH <- 2.5
GRAPH_TWO_COLUMN_WIDTH <- 9

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  # "metrics_valuation_rates.env_1_1726745431-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-19-12-15.tar.xz",
  # "metrics_valuation_rates.env_1_1726745431-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-19-11-55.tar.xz",
  # "metrics_valuation_rates.env_1_1726745431-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-19-12-36.tar.xz",
  # "metrics_valuation_rates.env_1_1726745431-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-19-12-57.tar.xz",
  # "metrics_valuation_rates.env_1_1726752700-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-19-15-24.tar.xz",
  # "metrics_valuation_rates.env_1_1726752700-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-19-14-48.tar.xz",
  # "metrics_valuation_rates.env_1_1726752700-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-19-16-00.tar.xz",
  # "metrics_valuation_rates.env_1_1726752700-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-19-14-11.tar.xz",
  # "metrics_valuation_rates.env_1_1726835168-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-20-13-18.tar.xz",
  # "metrics_valuation_rates.env_1_1726835168-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-20-13-01.tar.xz",
  # "metrics_valuation_rates.env_1_1726835168-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-20-13-51.tar.xz",
  # "metrics_valuation_rates.env_1_1726835168-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-20-13-35.tar.xz",
  #---
  # "metrics_valuation_rates.env_1_1727084593-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-11-40.tar.xz",
  # # "metrics_valuation_rates.env_1_1727084593-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-23-11-21.tar.xz",
  # "metrics_valuation_rates.env_1_1727084593-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-11-04.tar.xz",
  # "metrics_valuation_rates.env_1_1727084593-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-10-29.tar.xz",
  # "metrics_valuation_rates.env_1_1727084593-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-10-47.tar.xz",
  # "metrics_valuation_rates.env_1_1727084593-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-23-11-59.tar.xz",
  # "metrics_valuation_rates.env_1_1727095102-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-13-38.tar.xz",
  # # "metrics_valuation_rates.env_1_1727095102-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-23-13-09.tar.xz",
  # "metrics_valuation_rates.env_1_1727095102-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-14-07.tar.xz",
  # "metrics_valuation_rates.env_1_1727095102-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-13-52.tar.xz",
  # "metrics_valuation_rates.env_1_1727095102-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-23-12-55.tar.xz",
  # "metrics_valuation_rates.env_1_1727095102-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-23-13-24.tar.xz",
  # "metrics_valuation_rates.env_1_1727163428-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-24-07-54.tar.xz",
  # # "metrics_valuation_rates.env_1_1727163428-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-24-08-39.tar.xz",
  # "metrics_valuation_rates.env_1_1727163428-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-24-08-54.tar.xz",
  # "metrics_valuation_rates.env_1_1727163428-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-24-09-09.tar.xz",
  # "metrics_valuation_rates.env_1_1727163428-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-24-08-24.tar.xz",
  # "metrics_valuation_rates.env_1_1727163428-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-24-08-09.tar.xz",
  # "metrics_valuation_rates.env_1_1727249409-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-25-08-42.tar.xz",
  # # "metrics_valuation_rates.env_1_1727249409-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-25-09-57.tar.xz",
  # "metrics_valuation_rates.env_1_1727249409-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-25-10-24.tar.xz",
  # "metrics_valuation_rates.env_1_1727249409-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-25-09-33.tar.xz",
  # "metrics_valuation_rates.env_1_1727249409-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-25-09-10.tar.xz",
  # "metrics_valuation_rates.env_1_1727249409-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-25-10-47.tar.xz",
  # "metrics_valuation_rates.env_1_1727339260-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-08-59.tar.xz",
  # # "metrics_valuation_rates.env_1_1727339260-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-26-09-58.tar.xz",
  # "metrics_valuation_rates.env_1_1727339260-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-09-15.tar.xz",
  # "metrics_valuation_rates.env_1_1727339260-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-08-45.tar.xz",
  # "metrics_valuation_rates.env_1_1727339260-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-09-44.tar.xz",
  # "metrics_valuation_rates.env_1_1727339260-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-26-09-29.tar.xz",
  # "metrics_valuation_rates.env_1_1727346604-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-11-23.tar.xz",
  # # "metrics_valuation_rates.env_1_1727346604-fog_node-auction-quadratic_rates-reduction-market-default_strategy-.env.1_2024-09-26-11-08.tar.xz",
  # "metrics_valuation_rates.env_1_1727346604-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-11-54.tar.xz",
  # "metrics_valuation_rates.env_1_1727346604-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-11-39.tar.xz",
  # "metrics_valuation_rates.env_1_1727346604-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-12-10.tar.xz",
  # "metrics_valuation_rates.env_1_1727346604-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-26-10-52.tar.xz",
  # "metrics_valuation_rates.env_1_1727374375-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-21-46.tar.xz",
  # "metrics_valuation_rates.env_1_1727374375-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-21-10.tar.xz",
  # "metrics_valuation_rates.env_1_1727374375-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-20-51.tar.xz",
  # "metrics_valuation_rates.env_1_1727374375-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-21-28.tar.xz",
  # "metrics_valuation_rates.env_1_1727374375-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-26-20-33.tar.xz",
  # "metrics_valuation_rates.env_2_1727374375-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-20-18.tar.xz",
  # "metrics_valuation_rates.env_2_1727374375-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-20-34.tar.xz",
  # "metrics_valuation_rates.env_2_1727374375-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-20-48.tar.xz",
  # "metrics_valuation_rates.env_2_1727374375-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-20-04.tar.xz",
  # "metrics_valuation_rates.env_2_1727374375-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-26-21-03.tar.xz",
  # "metrics_valuation_rates.env_3_1727374375-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-22-13.tar.xz",
  # "metrics_valuation_rates.env_3_1727374375-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-21-01.tar.xz",
  # "metrics_valuation_rates.env_3_1727374375-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-21-25.tar.xz",
  # "metrics_valuation_rates.env_3_1727374375-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-26-22-43.tar.xz",
  # "metrics_valuation_rates.env_3_1727374375-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-26-21-50.tar.xz",
  # "metrics_valuation_rates.env_1_1727678830-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-07-41.tar.xz",
  # "metrics_valuation_rates.env_1_1727678830-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-09-00.tar.xz",
  # "metrics_valuation_rates.env_1_1727678830-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-08-22.tar.xz",
  # "metrics_valuation_rates.env_1_1727678830-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-08-41.tar.xz",
  # "metrics_valuation_rates.env_1_1727678830-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-30-08-02.tar.xz",
  # "metrics_valuation_rates.env_1_1727691307-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-11-36.tar.xz",
  # "metrics_valuation_rates.env_1_1727691307-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-11-21.tar.xz",
  # "metrics_valuation_rates.env_1_1727691307-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-11-07.tar.xz",
  # "metrics_valuation_rates.env_1_1727691307-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-09-30-10-51.tar.xz",
  # "metrics_valuation_rates.env_1_1727691307-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-09-30-10-37.tar.xz",
  # "metrics_valuation_rates.env_1_1727775765-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-01-11-03.tar.xz",
  # "metrics_valuation_rates.env_1_1727775765-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-01-11-36.tar.xz",
  # "metrics_valuation_rates.env_1_1727775765-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-01-11-20.tar.xz",
  # "metrics_valuation_rates.env_1_1727775765-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-01-11-52.tar.xz",
  # "metrics_valuation_rates.env_1_1727775765-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-01-10-47.tar.xz",
  # "metrics_valuation_rates.env_1_1727960423-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-03-14-57.tar.xz",
  # "metrics_valuation_rates.env_1_1727960423-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-03-14-18.tar.xz",
  # "metrics_valuation_rates.env_1_1727960423-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-03-15-19.tar.xz",
  # "metrics_valuation_rates.env_1_1727960423-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-03-13-58.tar.xz",
  # "metrics_valuation_rates.env_1_1727960423-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-03-14-37.tar.xz",
  # "metrics_valuation_rates.env_1_1728027062-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-04-08-12.tar.xz",
  # "metrics_valuation_rates.env_1_1728027062-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-04-07-57.tar.xz",
  # "metrics_valuation_rates.env_1_1728027062-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-04-08-27.tar.xz",
  # "metrics_valuation_rates.env_1_1728027062-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-04-08-43.tar.xz",
  # "metrics_valuation_rates.env_1_1728027062-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-04-08-59.tar.xz",
  # "metrics_valuation_rates.env_1_1728304420-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-07-13-31.tar.xz",
  # "metrics_valuation_rates.env_1_1728304420-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-07-13-15.tar.xz",
  # "metrics_valuation_rates.env_1_1728304420-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-07-12-59.tar.xz",
  # "metrics_valuation_rates.env_1_1728304420-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-07-13-46.tar.xz",
  # "metrics_valuation_rates.env_1_1728304420-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-07-14-02.tar.xz",
  # "metrics_valuation_rates.env_1_1728482274-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-09-15-59.tar.xz",
  # "metrics_valuation_rates.env_1_1728482274-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-09-16-52.tar.xz",
  # "metrics_valuation_rates.env_1_1728482274-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-09-15-36.tar.xz",
  # "metrics_valuation_rates.env_1_1728482274-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-09-16-28.tar.xz",
  # "metrics_valuation_rates.env_1_1728482274-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-09-15-12.tar.xz",
  # "metrics_valuation_rates.env_1_1728632352-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-11-09-00.tar.xz",
  # "metrics_valuation_rates.env_1_1728632352-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-11-10-23.tar.xz",
  # "metrics_valuation_rates.env_1_1728632352-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-11-09-32.tar.xz",
  # "metrics_valuation_rates.env_1_1728632352-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-11-09-58.tar.xz",
  # "metrics_valuation_rates.env_1_1728632352-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-11-10-48.tar.xz",
  #---
  # "metrics_valuation_rates.env_DEV-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-18-14-15.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-18-15-49.tar.xz",
  ## "metrics_valuation_rates.env_1_1729494677-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-07-30.tar.xz",
  ## "metrics_valuation_rates.env_1_1729494677-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-08-34.tar.xz",
  # "metrics_valuation_rates.env_1_1729494677-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-08-18.tar.xz",
  # "metrics_valuation_rates.env_1_1729494677-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-08-02.tar.xz",
  # "metrics_valuation_rates.env_1_1729494677-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-21-07-47.tar.xz",
  # "metrics_valuation_rates.env_1_1729505187-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-11-26.tar.xz",
  # "metrics_valuation_rates.env_1_1729505187-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-10-57.tar.xz",
  # "metrics_valuation_rates.env_1_1729505187-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-10-27.tar.xz",
  # "metrics_valuation_rates.env_1_1729505187-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-21-10-43.tar.xz",
  # "metrics_valuation_rates.env_1_1729505187-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-21-11-12.tar.xz",
  #  "metrics_valuation_rates.env_1_1729605644-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-22-14-36.tar.xz",
  #  "metrics_valuation_rates.env_1_1729605644-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-22-14-24.tar.xz",
  #  "metrics_valuation_rates.env_1_1729605644-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-22-14-30.tar.xz",
  #  "metrics_valuation_rates.env_1_1729605644-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-22-14-11.tar.xz",
  #  "metrics_valuation_rates.env_1_1729605644-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-22-14-17.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-23-07-54.tar.xz",
  #  "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-23-08-54.tar.xz",

  #--
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-23-09-55.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-23-12-21.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-23-12-52.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-23-13-34.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev_2024-10-23-15-32.tar.xz",
  #---
  "metrics_valuation_rates.env_1_1729755003-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-08-34.tar.xz",
  "metrics_valuation_rates.env_1_1729755003-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-08-03.tar.xz",
  "metrics_valuation_rates.env_1_1729755003-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-07-47.tar.xz",
  "metrics_valuation_rates.env_1_1729755003-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-08-18.tar.xz",
  "metrics_valuation_rates.env_2_1729755003-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-08-38.tar.xz",
  "metrics_valuation_rates.env_2_1729755003-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-08-22.tar.xz",
  "metrics_valuation_rates.env_2_1729755003-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-07-52.tar.xz",
  "metrics_valuation_rates.env_2_1729755003-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-24-08-07.tar.xz",
  "metrics_valuation_rates.env_1_1729755003-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-24-08-49.tar.xz",
  "metrics_valuation_rates.env_2_1729755003-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-08-53.tar.xz",
  "metrics_valuation_rates.env_1_1729760444-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-09-24.tar.xz",
  "metrics_valuation_rates.env_1_1729760444-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-10-28.tar.xz",
  "metrics_valuation_rates.env_1_1729760444-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-09-40.tar.xz",
  "metrics_valuation_rates.env_1_1729760444-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-09-56.tar.xz",
  "metrics_valuation_rates.env_1_1729760444-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-24-10-12.tar.xz",
  "metrics_valuation_rates.env_1_1729768662-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-11-48.tar.xz",
  "metrics_valuation_rates.env_1_1729768662-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-12-04.tar.xz",
  "metrics_valuation_rates.env_1_1729768662-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-24-12-21.tar.xz",
  "metrics_valuation_rates.env_1_1729768662-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-12-37.tar.xz",
  "metrics_valuation_rates.env_1_1729768662-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-12-53.tar.xz",
  "metrics_valuation_rates.env_1_1729775693-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-14-13.tar.xz",
  "metrics_valuation_rates.env_1_1729775693-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-13-54.tar.xz",
  "metrics_valuation_rates.env_1_1729775693-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-14-52.tar.xz",
  "metrics_valuation_rates.env_1_1729775693-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-24-15-10.tar.xz",
  "metrics_valuation_rates.env_1_1729775693-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-24-14-33.tar.xz",
  "metrics_valuation_rates.env_1_1729841561-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-25-08-24.tar.xz",
  "metrics_valuation_rates.env_1_1729841561-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-25-08-51.tar.xz",
  "metrics_valuation_rates.env_1_1729841561-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-25-09-13.tar.xz",
  "metrics_valuation_rates.env_1_1729841561-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-25-09-57.tar.xz",
  "metrics_valuation_rates.env_1_1729841561-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-25-09-35.tar.xz",
  "metrics_valuation_rates.env_1_1730113032-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-11-48.tar.xz",
  "metrics_valuation_rates.env_1_1730113032-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-12-22.tar.xz",
  "metrics_valuation_rates.env_1_1730113032-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-11-31.tar.xz",
  "metrics_valuation_rates.env_1_1730113032-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-12-39.tar.xz",
  "metrics_valuation_rates.env_1_1730113032-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-28-12-05.tar.xz",
  "metrics_valuation_rates.env_1_1730120762-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-14-22.tar.xz",
  "metrics_valuation_rates.env_1_1730120762-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-13-50.tar.xz",
  "metrics_valuation_rates.env_1_1730120762-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-14-39.tar.xz",
  "metrics_valuation_rates.env_1_1730120762-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-28-13-33.tar.xz",
  "metrics_valuation_rates.env_1_1730120762-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-28-14-06.tar.xz",
  "metrics_valuation_rates.env_1_1730186960-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-29-09-48.tar.xz",
  "metrics_valuation_rates.env_1_1730186960-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-29-09-16.tar.xz",
  "metrics_valuation_rates.env_1_1730186960-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-29-11-05.tar.xz",
  "metrics_valuation_rates.env_1_1730186960-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-10-29-08-44.tar.xz",
  "metrics_valuation_rates.env_1_1730186960-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-10-29-10-32.tar.xz",
  "metrics_valuation_rates.env_1_1730712248-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-12-52.tar.xz",
  "metrics_valuation_rates.env_1_1730712248-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-11-16.tar.xz",
  "metrics_valuation_rates.env_1_1730712248-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-12-21.tar.xz",
  "metrics_valuation_rates.env_1_1730712248-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-10-44.tar.xz",
  "metrics_valuation_rates.env_1_1730712248-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-11-04-11-48.tar.xz",
  "metrics_valuation_rates.env_1_1730742431-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-21-57.tar.xz",
  "metrics_valuation_rates.env_1_1730742431-fog_node-edge_first-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-20-45.tar.xz",
  "metrics_valuation_rates.env_1_1730742431-fog_node-edge_furthest-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-21-21.tar.xz",
  "metrics_valuation_rates.env_1_1730742431-fog_node-edge_ward-quadratic_rates-no_complication-market-default_strategy-.env.1_2024-11-04-21-39.tar.xz",
  "metrics_valuation_rates.env_1_1730742431-fog_node-mincpurandom-quadratic_rates-no_complication-market-mincpurandom-.env.1_2024-11-04-21-03.tar.xz",

  #
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
options(width = 10000)

# Not scientific notations
options(scipen = 10000)

options(error = function() {
  calls <- sys.calls()
  if (length(calls) >= 2L) {
    sink(stderr())
    on.exit(sink(NULL))
    cat("Backtrace:\n")
    calls <- rev(calls[-length(calls)])
    for (i in seq_along(calls)) {
      cat(i, ": ", deparse(calls[[i]], nlines = 1L), "\n", sep = "")
    }
  }
  if (!interactive()) {
    q(status = 1)
  }
})
