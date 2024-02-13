generate_gif <- T
reload_big_data <- TRUE
parallel_loading_datasets <- 2
parallel_loading_datasets_small <- 22
time_interval <- 15 # secs


GRAPH_ONE_COLUMN_HEIGHT <- 3
GRAPH_ONE_COLUMN_WIDTH <- 5
GRAPH_HALF_COLUMN_WIDTH <- 2.5
GRAPH_TWO_COLUMN_WIDTH <- 12

METRICS_PATH <- "../metrics-arks"
METRICS_ARKS <- c(
    "metrics_valuation_rates.env_1-auction_valuation_rates_no-telemetry_2024-02-02-16-18.tar.xz",
    # "metrics_valuation_rates.env_1-edge_ward_valuation_rates_no-telemetry_2024-02-02-16-18.tar.xz",
    #---
    #---
    #---
    "last element that is here because last element should not have any comma in the end and that sucks hard time."
)
METRICS_ARKS <- METRICS_ARKS[-length(METRICS_ARKS)]

METRICS_GROUP <- str_match(METRICS_ARKS, "metrics_.*-(.*?)_valuation.*\\.tar\\.xz")
METRICS_GROUP <- METRICS_GROUP[, 2]
METRICS_GROUP_GROUP <- as.character(rep(list("toto"), length(METRICS_ARKS)))

length(METRICS_ARKS)
length(METRICS_GROUP)
length(METRICS_GROUP_GROUP)
stopifnot(length(METRICS_ARKS) == length(METRICS_GROUP))
stopifnot(length(METRICS_ARKS) == length(METRICS_GROUP_GROUP))

# Make the output of the console real wide
# alt+z on the vscode console to make it not wrap
options(width = 1000)
