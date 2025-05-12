generate_gif <- FALSE
reload_big_data <- FALSE
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

  #---
  #---
  #---
  "last element that is here because last element should not have any comma in the end"
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
