generate_gif <- FALSE
reload_big_data <- FALSE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
all_workers <- parallel::detectCores()
workers <- min(all_workers, 8)
time_interval <- 15 # secs

CHAIN_LENGTH <- 3

# GRAPH_ONE_COLUMN_HEIGHT <- 3 * 1.5
GRAPH_ONE_COLUMN_HEIGHT <- 2 * 2
GRAPH_ONE_COLUMN_WIDTH <- 3.6 * 2
GRAPH_HALF_COLUMN_WIDTH <- 2.5 * 2
GRAPH_TWO_COLUMN_WIDTH <- 6 * 2

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  # "metrics_valuation_rates.env_1_1759856614-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-07-17-47.tar.xz",
  # "metrics_valuation_rates.env_1_1759856614-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-07-17-29.tar.xz",
  #
  # "metrics_valuation_rates.env_1_1759870902-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-07-21-46.tar.xz",
  # "metrics_valuation_rates.env_1_1759870902-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-07-22-42.tar.xz",
  # "metrics_valuation_rates.env_1_1759870902-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-10-07-21-27.tar.xz",
  # "metrics_valuation_rates.env_1_1759870902-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-10-07-22-04.tar.xz",
  # "metrics_valuation_rates.env_1_1759870902-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-10-07-23-01.tar.xz",
  # "metrics_valuation_rates.env_1_1759870902-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-10-07-22-25.tar.xz",
  #
  # "metrics_valuation_rates.env_1_1759908316-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-08-07-50.tar.xz",
  # "metrics_valuation_rates.env_1_1759908316-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-08-08-46.tar.xz",
  # "metrics_valuation_rates.env_1_1759908316-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-08-08-09.tar.xz",
  # "metrics_valuation_rates.env_1_1759908316-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-08-08-27.tar.xz",

  # "metrics_valuation_rates.env_1_1759921165-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-08-11-52.tar.xz",
  # "metrics_valuation_rates.env_1_1759921165-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-08-11-34.tar.xz",
  # "metrics_valuation_rates.env_1_1759921165-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-08-12-10.tar.xz",

  # "metrics_valuation_rates.env_1_1759927358-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-08-14-03.tar.xz",
  # "metrics_valuation_rates.env_1_1759927358-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-08-13-10.tar.xz",
  # "metrics_valuation_rates.env_1_1759927358-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-08-13-28.tar.xz",
  # "metrics_valuation_rates.env_1_1759927358-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-08-13-46.tar.xz",
  #
  # "metrics_valuation_rates.env_1_1759933678-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-08-15-47.tar.xz",
  # "metrics_valuation_rates.env_1_1759933678-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-08-15-12.tar.xz",
  # "metrics_valuation_rates.env_1_1759933678-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-08-15-29.tar.xz",
  # "metrics_valuation_rates.env_1_1759933678-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-08-14-53.tar.xz",

  # "metrics_valuation_rates.env_1_1760080818-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-10-07-57.tar.xz",
  # "metrics_valuation_rates.env_1_1760080818-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-10-08-21.tar.xz",

  # "metrics_valuation_rates.env_1_1760001330-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-09-09-58.tar.xz",
  # "metrics_valuation_rates.env_1_1760001330-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-09-10-32.tar.xz",
  # "metrics_valuation_rates.env_1_1760001330-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-09-09-39.tar.xz",

  # "metrics_valuation_rates.env_1_1760110495-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-10-15-57.tar.xz",
  "metrics_valuation_rates.env_1_1760427270-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-14-07-55.tar.xz",
  "metrics_valuation_rates.env_1_1760427270-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-14-08-14.tar.xz",
  "metrics_valuation_rates.env_1_1760437039-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-14-10-49.tar.xz",
  "metrics_valuation_rates.env_1_1760437039-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-14-11-10.tar.xz",

  "metrics_valuation_rates.env_1_1760446247-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-14-13-30.tar.xz",
  "metrics_valuation_rates.env_1_1760446247-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-14-14-32.tar.xz",
  "metrics_valuation_rates.env_1_1760446247-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-14-14-05.tar.xz",

  "metrics_valuation_rates.env_1_1760461099-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-14-18-30.tar.xz",
  "metrics_valuation_rates.env_1_1760461099-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-14-18-09.tar.xz",
  "metrics_valuation_rates.env_1_1760461099-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-14-17-27.tar.xz",
  "metrics_valuation_rates.env_1_1760461099-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-14-17-49.tar.xz",

  "metrics_valuation_rates.env_1_1760468453-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-14-20-17.tar.xz",
  "metrics_valuation_rates.env_1_1760468453-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-10-14-19-41.tar.xz",
  "metrics_valuation_rates.env_1_1760476125-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-14-22-02.tar.xz",
  "metrics_valuation_rates.env_1_1760476125-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-14-21-39.tar.xz",
  "metrics_valuation_rates.env_1_1760476125-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-14-22-53.tar.xz",
  "metrics_valuation_rates.env_1_1760476125-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-10-14-23-15.tar.xz",
  "metrics_valuation_rates.env_1_1760476125-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-10-15-00-07.tar.xz",
  "metrics_valuation_rates.env_1_1760476125-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-10-15-00-35.tar.xz",

  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-15-09-18.tar.xz",
  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-15-09-55.tar.xz",
  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-15-09-00.tar.xz",
  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-15-08-22.tar.xz",
  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-10-15-09-36.tar.xz",
  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-10-15-08-40.tar.xz",
  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-10-15-08-03.tar.xz",
  "metrics_valuation_rates.env_1_1760514228-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-10-15-10-13.tar.xz",

  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-15-14-37.tar.xz",
  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-15-13-19.tar.xz",
  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-15-13-59.tar.xz",
  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-10-15-12-19.tar.xz",
  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-10-15-13-38.tar.xz",
  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-10-15-14-18.tar.xz",
  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-10-15-12-41.tar.xz",
  "metrics_valuation_rates.env_1_1760529222-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-15-13-00.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-10-15-17-37.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-10-15-16-18.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-10-15-16-00.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-10-15-17-17.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-10-15-16-57.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-10-15-15-41.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-10-15-16-37.tar.xz",
  "metrics_valuation_rates.env_1_1760561021-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-15-22-02.tar.xz",
  "metrics_valuation_rates.env_1_1760561021-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-10-15-21-28.tar.xz",
  "metrics_valuation_rates.env_1_1760561021-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-10-15-23-08.tar.xz",
  "metrics_valuation_rates.env_1_1760561021-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-10-15-22-31.tar.xz",
  "metrics_valuation_rates.env_1_1760540370-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-10-15-15-22.tar.xz",
  #---
  #---
  #---
  "last element that is here because last element should not have any comma in the end"
)
METRICS_ARKS <- METRICS_ARKS[-length(METRICS_ARKS)]


env_live_extract <- function(x) {
  order <- c(
    "10u/req, $\\infty$ initial budget, No fallbacks",
    "10u/req, $\\infty$ initial budget",
    "10u/req",
    "20u/req"
  )

  x %>%
    mutate(
      # scenrios relevant for pressure
      pressure = env_live <= 2
    ) %>%
    mutate(
      env_live = case_when(
        env_live == 1 ~ "10u/req, $\\infty$ initial budget, No fallbacks",
        env_live == 2 ~ "10u/req, $\\infty$ initial budget",
        env_live == 3 ~ "10u/req",
        env_live == 4 ~ "20u/req"
      ),
      env_live = factor(env_live, levels = order)
    )
}

extract_function_name <- function(spans) {
  order <- c(
    "Speech to Text",
    "Speech to Text (degraded)",
    "Sentiment Analysis",
    "Text to Speech",
    "End Function"
  )

  spans %>%
    mutate(
      span.name = case_when(
        span.name == "SpeechToText" ~ "Speech to Text",
        span.name == "VoskSpeechToText" ~ "Speech to Text (degraded)",
        span.name == "Sentiment" ~ "Sentiment Analysis",
        span.name == "EndGame" ~ "End Function",
        span.name == "TextToSpeech" ~ "Text to Speech",
        TRUE ~ paste0(span.name, " (raw)")
      ),
      span.name = factor(span.name, levels = order)
    )
}

extract_env_name <- function(x) {
  x %>%
    mutate(
      env = case_when(
        env == 1 ~ "High load",
        env == 2 ~ "Normal load",
        env == 3 ~ "Normal load, more functions",
        TRUE ~ env
      )
    )
}

categorize_nb_nodes <- function(nb_nodes) {
  nb_nodes %>%
    mutate(
      nb_nodes = case_when(
        nb_nodes < 100 ~ "$<100$",
        nb_nodes < 200 ~ "$100\\leq x<200$",
        nb_nodes < 300 ~ "$200\\leq x<300$",
        nb_nodes < 400 ~ "$300\\leq x<400$",
        TRUE ~ as.character(nb_nodes)
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
