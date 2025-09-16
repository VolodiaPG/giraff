generate_gif <- FALSE
reload_big_data <- FALSE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
all_workers <- parallel::detectCores()
workers <- min(all_workers, 6)
time_interval <- 15 # secs

no_memoization <- FALSE
single_graphs <- TRUE

CHAIN_LENGTH <- 3

GRAPH_ONE_COLUMN_HEIGHT <- 3 * 1.5
GRAPH_ONE_COLUMN_WIDTH <- 3.6 * 1.5
GRAPH_HALF_COLUMN_WIDTH <- 2.5 * 1.5
GRAPH_TWO_COLUMN_WIDTH <- 6 * 1.5

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  # "metrics_valuation_rates.env_1_1756289382-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-08-27-11-01.tar.xz",
  # "metrics_valuation_rates.env_1_1756289382-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-08-27-10-30.tar.xz",
  # "metrics_valuation_rates.env_1_1756289382-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-08-27-10-46.tar.xz",
  # "metrics_valuation_rates.env_1_1756289382-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-08-27-11-17.tar.xz",
  # ---
  # "metrics_valuation_rates.env_1_1756385360-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-08-28-13-08.tar.xz",
  # "metrics_valuation_rates.env_1_1756385360-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-08-28-13-24.tar.xz",
  # "metrics_valuation_rates.env_1_1756385360-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-08-28-13-59.tar.xz",
  # "metrics_valuation_rates.env_1_1756385360-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-08-28-13-42.tar.xz",
  # ---
  # "metrics_valuation_rates.env_1_1756458124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-08-29-10-07.tar.xz",
  # "metrics_valuation_rates.env_1_1756458124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-08-29-09-37.tar.xz",
  # "metrics_valuation_rates.env_1_1756458124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-08-29-09-21.tar.xz",
  # "metrics_valuation_rates.env_1_1756458124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-08-29-09-52.tar.xz",
  # ---
  # "metrics_valuation_rates.env_1_1756481124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-08-29-16-22.tar.xz",
  # "metrics_valuation_rates.env_1_1756481124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-08-29-16-07.tar.xz",
  # "metrics_valuation_rates.env_1_1756481124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-08-29-16-39.tar.xz",
  # "metrics_valuation_rates.env_1_1756481124-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-08-29-15-50.tar.xz",

  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-02-07-25.tar.xz",
  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-02-07-40.tar.xz",
  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-02-06-41.tar.xz",
  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-02-07-55.tar.xz",
  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-02-06-55.tar.xz",
  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-02-07-10.tar.xz",
  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-02-08-25.tar.xz",
  # "metrics_valuation_rates.env_1_1756794209-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-02-08-10.tar.xz",

  # "metrics_valuation_rates.env_1_1757330683-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-08-11-42.tar.xz",
  # "metrics_valuation_rates.env_1_1757334236-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-08-12-38.tar.xz",
  # "metrics_valuation_rates.env_1_1757334236-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-08-12-53.tar.xz",

  # "metrics_valuation_rates.env_1_1757339268-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-08-14-11.tar.xz",
  # "metrics_valuation_rates.env_1_1757339268-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-08-14-45.tar.xz",
  # "metrics_valuation_rates.env_1_1757339268-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-08-15-32.tar.xz",
  # "metrics_valuation_rates.env_1_1757339268-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-08-14-29.tar.xz",
  # "metrics_valuation_rates.env_1_1757339268-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-08-15-01.tar.xz",
  # "metrics_valuation_rates.env_1_1757339268-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-08-15-17.tar.xz",

  # "metrics_valuation_rates.env_1_1757417621-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-09-12-28.tar.xz",
  # "metrics_valuation_rates.env_1_1757417621-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-09-12-45.tar.xz",
  # "metrics_valuation_rates.env_1_1757417621-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-09-12-11.tar.xz",
  # "metrics_valuation_rates.env_1_1757417621-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-09-11-56.tar.xz",

  # "metrics_valuation_rates.env_1_1757427102-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-09-15-44.tar.xz",
  # "metrics_valuation_rates.env_1_1757427102-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-09-14-36.tar.xz",
  # "metrics_valuation_rates.env_1_1757427102-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-09-15-00.tar.xz",
  # "metrics_valuation_rates.env_1_1757427102-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-09-15-22.tar.xz",

  # "metrics_valuation_rates.env_1_1757489004-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-10-08-29.tar.xz",
  # "metrics_valuation_rates.env_1_1757489004-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-10-08-50.tar.xz",
  # "metrics_valuation_rates.env_1_1757489004-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-10-07-48.tar.xz",
  # "metrics_valuation_rates.env_1_1757489004-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-10-08-09.tar.xz",

  # "metrics_valuation_rates.env_1_1757505843-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-10-13-18.tar.xz",
  # "metrics_valuation_rates.env_1_1757505843-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-10-12-34.tar.xz",
  # "metrics_valuation_rates.env_1_1757505843-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-10-13-40.tar.xz",
  # "metrics_valuation_rates.env_1_1757505843-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-10-12-56.tar.xz",

  # "metrics_valuation_rates.env_1_1757519854-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-10-16-40.tar.xz",
  # "metrics_valuation_rates.env_1_1757519854-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-10-16-58.tar.xz",
  # "metrics_valuation_rates.env_1_1757519854-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-10-16-22.tar.xz",
  # "metrics_valuation_rates.env_1_1757519854-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-10-17-14.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev-.env.live.dev_2025-09-11-08-01.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev-.env.live.dev_2025-09-11-19-02.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev-.env.live.dev_2025-09-11-20-07.tar.xz",
  # "metrics_valuation_rates.env_1_1757666042-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-12-09-03.tar.xz",

  # "metrics_valuation_rates.env_1_1757668543-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-12-09-59.tar.xz",
  # "metrics_valuation_rates.env_1_1757668543-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-12-09-42.tar.xz",
  # "metrics_valuation_rates.env_1_1757668543-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-12-10-17.tar.xz",
  # #
  # "metrics_valuation_rates.env_1_1757679129-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-12-12-50.tar.xz",
  # "metrics_valuation_rates.env_1_1757679129-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-12-12-35.tar.xz",

  # "metrics_valuation_rates.env_1_1757681960-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-12-13-34.tar.xz",
  # "metrics_valuation_rates.env_1_1757681960-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-12-13-53.tar.xz",

  # "metrics_valuation_rates.env_1_1757713666-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-12-22-45.tar.xz",
  # "metrics_valuation_rates.env_1_1757713666-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-12-22-27.tar.xz",

  # "metrics_valuation_rates.env_1_1757924915-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-15-09-27.tar.xz",
  # "metrics_valuation_rates.env_1_1757924915-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-15-08-54.tar.xz",
  # "metrics_valuation_rates.env_1_1757924915-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-15-09-11.tar.xz",
  # "metrics_valuation_rates.env_1_1757924915-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-15-09-43.tar.xz",

  # "metrics_valuation_rates.env_1_1757937582-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-15-13-20.tar.xz",
  # "metrics_valuation_rates.env_1_1757937582-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-15-13-02.tar.xz",
  # "metrics_valuation_rates.env_1_1757937582-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-15-12-26.tar.xz",
  # "metrics_valuation_rates.env_1_1757937582-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-15-12-44.tar.xz",
  # "metrics_valuation_rates.env_DEV-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.dev-.env.live.dev_2025-09-16-09-29.tar.xz",

  "metrics_valuation_rates.env_1_1758026895-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-16-13-14.tar.xz",
  "metrics_valuation_rates.env_1_1758026895-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-16-13-32.tar.xz",
  #---
  #---
  #---
  "last element that is here because last element should not have any comma in the end"
)
METRICS_ARKS <- METRICS_ARKS[-length(METRICS_ARKS)]


env_live_extract <- function(x) {
  x %>%
    mutate(
      env_live = case_when(
        env_live == 1 ~ "No fallbacks",
        env_live == 2 ~ "25u/req",
        env_live == 3 ~ "50u/req"
      )
    )
}


library(stringr)
library(digest)

METRICS_ARKS_DF <- data.frame(
  name = sapply(METRICS_ARKS, function(x) {
    digest(as.character(x), algo = "md5")
  }),
  ark = METRICS_ARKS,
  stringsAsFactors = FALSE
)

METRICS_GROUP <- str_match(
  METRICS_ARKS,
  "metrics_.*-fog_node-(.*)-market.*\\.tar\\.xz"
)
METRICS_GROUP <- METRICS_GROUP[, 2]
METRICS_GROUP_GROUP <- str_match(
  METRICS_ARKS,
  "metrics_.*\\.(.*)-fog_node-.*-market.*\\.tar\\.xz"
)
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
options(scipen = 9999)

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
