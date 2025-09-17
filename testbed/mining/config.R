generate_gif <- FALSE
reload_big_data <- FALSE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
all_workers <- parallel::detectCores()
workers <- min(all_workers, 6)
time_interval <- 15 # secs

CHAIN_LENGTH <- 3

# GRAPH_ONE_COLUMN_HEIGHT <- 3 * 1.5
GRAPH_ONE_COLUMN_HEIGHT <- 2 * 1.5
GRAPH_ONE_COLUMN_WIDTH <- 3.6 * 1.5
GRAPH_HALF_COLUMN_WIDTH <- 2.5 * 1.5
GRAPH_TWO_COLUMN_WIDTH <- 6 * 1.5

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  #   "metrics_valuation_rates.env_1_1758052759-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-16-20-29.tar.xz",
  # "metrics_valuation_rates.env_1_1758052759-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-16-20-48.tar.xz",
  # # "metrics_valuation_rates.env_1_1758052759-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-16-21-40.tar.xz",
  # "metrics_valuation_rates.env_1_1758052759-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-16-21-22.tar.xz",
  # "metrics_valuation_rates.env_1_1758052759-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-16-21-05.tar.xz",
  # "metrics_valuation_rates.env_1_1758052759-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-16-21-57.tar.xz",
  #
  # "metrics_valuation_rates.env_1_1758091583-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-17-07-20.tar.xz",
  # "metrics_valuation_rates.env_1_1758091583-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-17-07-38.tar.xz",

  # "metrics_valuation_rates.env_1_1758095178-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-17-09-22.tar.xz",
  "metrics_valuation_rates.env_1_1758101806-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-17-10-02.tar.xz",
  "metrics_valuation_rates.env_1_1758101806-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-17-10-31.tar.xz",
  "metrics_valuation_rates.env_1_1758101806-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-17-10-16.tar.xz",
  #---
  #---
  #---
  "last element that is here because last element should not have any comma in the end"
)
METRICS_ARKS <- METRICS_ARKS[-length(METRICS_ARKS)]


env_live_extract <- function(x) {
  x %>%
    mutate(
      # scenrios relevant for pressure
      pressure = env_live <= 2
    ) %>%
    mutate(
      env_live = case_when(
        env_live == 1 ~ "$\\infty$u/req, No fallbacks",
        env_live == 2 ~ "$\\infty$u/req",
        env_live == 3 ~ "25u/req",
        env_live == 4 ~ "50u/req"
      )
    )
}

extract_function_name <- function(spans) {
  spans %>%
    mutate(
      span.name = case_when(
        span.name == "SpeechToText" ~ "Speech to Text",
        span.name == "VoskSpeechToText " ~ "Speech to Text (degraded)",
        span.name == "Sentiment" ~ "Sentiment Analysis",
        span.name == "EndGame" ~ "End Function",
        span.name == "TextToSpeech" ~ "Text to Speech",
        TRUE ~ span.name
      )
    )
}

extract_env_name <- function(x) {
  x %>%
    mutate(
      env = case_when(
        env == 1 ~ "Higher load",
        env == 2 ~ "Lower load"
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
