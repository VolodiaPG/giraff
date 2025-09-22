generate_gif <- FALSE
reload_big_data <- FALSE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
all_workers <- parallel::detectCores()
workers <- min(all_workers, 6)
time_interval <- 15 # secs

CHAIN_LENGTH <- 3

# GRAPH_ONE_COLUMN_HEIGHT <- 3 * 1.5
GRAPH_ONE_COLUMN_HEIGHT <- 2 * 2
GRAPH_ONE_COLUMN_WIDTH <- 3.6 * 2
GRAPH_HALF_COLUMN_WIDTH <- 2.5 * 2
GRAPH_TWO_COLUMN_WIDTH <- 6 * 2

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
  # "metrics_valuation_rates.env_1_1758112101-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-17-13-40.tar.xz",
  # "metrics_valuation_rates.env_1_1758112101-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-17-13-57.tar.xz",
  # "metrics_valuation_rates.env_1_1758112101-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-17-13-08.tar.xz",
  # "metrics_valuation_rates.env_1_1758112101-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-17-13-24.tar.xz",
  #
  # "metrics_valuation_rates.env_1_1758118542-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-17-15-02.tar.xz",
  # "metrics_valuation_rates.env_1_1758118542-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-17-15-34.tar.xz",
  # "metrics_valuation_rates.env_1_1758118542-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-17-15-52.tar.xz",
  # "metrics_valuation_rates.env_1_1758118542-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-17-15-18.tar.xz",
  #
  # "metrics_valuation_rates.env_1_1758134052-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-17-19-00.tar.xz",
  # "metrics_valuation_rates.env_1_1758134052-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-17-19-52.tar.xz",
  # "metrics_valuation_rates.env_1_1758134052-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-17-19-16.tar.xz",
  # "metrics_valuation_rates.env_1_1758134052-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-17-19-34.tar.xz",
  #
  # # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-17-21-50.tar.xz",
  # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-17-21-32.tar.xz",
  # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-17-22-24.tar.xz",
  # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-17-22-58.tar.xz",
  # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-17-22-40.tar.xz",
  # # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-17-23-16.tar.xz",
  # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-17-21-15.tar.xz",
  # "metrics_valuation_rates.env_2_1758141408-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-17-23-50.tar.xz",
  #
  # # "metrics_valuation_rates.env_2_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-18-08-51.tar.xz",
  # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-18-06-54.tar.xz",
  # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-18-07-36.tar.xz",
  # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-18-08-16.tar.xz",
  # # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-18-07-14.tar.xz",
  # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-18-06-18.tar.xz",
  # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-18-07-56.tar.xz",
  # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-18-09-57.tar.xz",
  # # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-18-09-34.tar.xz",
  # "metrics_valuation_rates.env_1_1758173067-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-18-09-14.tar.xz",

  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-18-23-52.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-18-23-18.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-18-23-01.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-18-22-11.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-18-22-27.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-18-22-45.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-18-23-35.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-18-21-38.tar.xz",

  # # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-19-00-47.tar.xz",
  # # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-19-00-26.tar.xz",
  # "metrics_valuation_rates.env_1_1758229722-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-19-00-09.tar.xz",
  # # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-19-07-37.tar.xz",
  # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-19-08-48.tar.xz",
  # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-19-08-14.tar.xz",
  # # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-19-09-07.tar.xz",
  # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-19-09-55.tar.xz",
  # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-19-09-38.tar.xz",
  # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-19-08-32.tar.xz",
  # # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-19-10-12.tar.xz",
  # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-19-07-00.tar.xz",
  # "metrics_valuation_rates.env_1_1758262974-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-19-07-18.tar.xz",

  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-19-11-13.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-19-11-30.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-19-12-07.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-19-12-25.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-19-12-41.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-19-11-50.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-19-13-01.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-19-13-33.tar.xz",
  # "metrics_valuation_rates.env_1_1758278869-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-19-13-50.tar.xz",

  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-19-15-39.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-19-15-57.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-19-16-32.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-19-16-15.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-19-17-28.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-19-15-03.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-19-17-45.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-19-17-10.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-19-15-22.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-19-16-52.tar.xz",
  # "metrics_valuation_rates.env_1_1758291872-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-19-18-18.tar.xz",

  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-20-19-21.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-20-21-27.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-20-22-04.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-20-22-42.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-20-20-17.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-20-21-45.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-20-22-23.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-20-20-52.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-20-19-40.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-20-20-35.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-20-20-00.tar.xz",
  # "metrics_valuation_rates.env_1_1758394342-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-20-21-09.tar.xz",

  # "metrics_valuation_rates.env_1_1758444817-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-21-09-28.tar.xz",
  # "metrics_valuation_rates.env_1_1758444817-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-21-09-48.tar.xz",

  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-21-13-48.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-21-13-13.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-21-14-42.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-21-15-18.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-21-14-07.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-21-14-24.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-21-12-54.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-21-15-35.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-21-15-52.tar.xz",
  "metrics_valuation_rates.env_1_1758457445-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-21-16-10.tar.xz",

  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.1_2025-09-21-19-04.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.2_2025-09-21-20-37.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-21-17-47.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-21-19-41.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-21-19-23.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-21-20-00.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-21-17-28.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.4_2025-09-21-18-27.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-21-17-09.tar.xz",
  "metrics_valuation_rates.env_1_1758472629-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.3_2025-09-21-18-09.tar.xz",

  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.2_2025-09-21-23-38.tar.xz",
  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.3_2025-09-22-00-36.tar.xz",
  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.1-.env.live.4_2025-09-22-01-15.tar.xz",
  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.1_2025-09-22-02-13.tar.xz",
  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.2_2025-09-22-01-35.tar.xz",
  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.2-.env.live.3_2025-09-22-01-54.tar.xz",
  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.1_2025-09-22-00-17.tar.xz",
  "metrics_valuation_rates.env_2_1758487362-fog_node-auction-quadratic_rates-no_complication-market-default_strategy-.env.3-.env.live.4_2025-09-22-02-32.tar.xz",

  #---
  #---
  #---
  "last element that is here because last element should not have any comma in the end"
)
METRICS_ARKS <- METRICS_ARKS[-length(METRICS_ARKS)]


env_live_extract <- function(x) {
  order <- c(
    "$\\infty$u/req, No fallbacks",
    "$\\infty$u/req",
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
        env_live == 1 ~ "$\\infty$u/req, No fallbacks",
        env_live == 2 ~ "$\\infty$u/req",
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
