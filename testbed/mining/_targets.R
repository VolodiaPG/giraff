# Created by use_targets().
# Follow the comments below to fill in this target script.
# Then follow the manual to check and run the pipeline:
#   https://books.ropensci.org/targets/walkthrough.html#inspect-the-pipeline

# To combine metrics, suffix them with _single in the map
# To export ggraphs, suffix them with _graph

# Load packages required to define the pipeline:
library(targets)
library(tarchetypes) # Load other packages as needed.
library(crew) # For parallel processing
library(dplyr)
library(tibble)
library(purrr)

# Source the configuration and utility files
source("config.R")
source("utils.R")
source("loaders.R")
source("commongraphs.R")
source("graphs.R")

Log("Starting targets")

# Set default target options with minimal packages
tar_option_set(
  packages = c(
    "tibble",
    "dplyr",
    "purrr",
    "stringr",
    "memoise",
    "ggplot2",
    "cowplot",
    "scales",
    "vroom",
    "zoo",
    "ggprism",
    "snakecase",
    "foreach",
    "doParallel",
    "multidplyr",
    "multcompView",
    "car",
    "plotly",
    "htmlwidgets",
    "htmltools",
    "jsonlite",
    "future",
    "rlang",
    "archive",
    "igraph",
    "formattable",
    "viridis",
    "patchwork",
    "ggbeeswarm",
    "tidyverse"
  ), # Packages that your targets need for their tasks.
  format = "qs", # Optionally set the default storage format. qs is fast.
  controller = crew::crew_controller_local(workers = all_workers), # Use workers from config.R
)

# metrics_arks <-
#   list(
#     # Target for METRICS_ARKS from config.R
#     tar_target(
#       name = metrics_arks,
#       command = METRICS_ARKS_DF
#     )
#   )

single_ops <- tar_map(
  # unlist = FALSE, # Return a nested list from tar_map()
  values = METRICS_ARKS_DF, # the value is named "ark"
  names = name,
  tar_target(
    name = node_levels_single,
    command = load_node_levels(ark)
  ),
  tar_target(
    name = bids_raw_single,
    command = load_bids_raw(ark)
  ),
  tar_target(
    name = provisioned_sla_single,
    command = load_provisioned_sla(ark)
  ),
  tar_target(
    name = functions_single,
    command = load_functions(ark)
  ),
  tar_target(
    name = raw_deployment_times_single,
    command = load_raw_deployment_times(ark)
  ),
  tar_target(
    name = raw_latency_single,
    command = load_raw_latency(ark)
  ),
  tar_target(
    name = otel_single,
    command = load_otel(ark)
  ),
  tar_target(
    name = bids_won_function_single,
    command = load_bids_won_function(bids_raw_single, provisioned_sla_single)
  ),
  tar_target(
    name = output_mean_time_to_deploy_simple_graph,
    command = output_mean_time_to_deploy_simple(raw_deployment_times_single)
  ),
  tar_target(
    name = output_loss_graph,
    command = output_loss(raw_latency_single)
  ),
  tar_target(
    name = output_raw_latency_graph,
    command = output_raw_latency(raw_latency_single)
  ),
  tar_target(
    name = output_otel_graph,
    command = output_otel_plot(otel_single)
  ),
  tar_target(
    name = output_otel_budget_graph,
    command = output_otel_budget_plot(otel_single)
  ),
  tar_target(
    name = output_otel_correlations_graph,
    command = output_otel_correlations_plot(otel_single, raw_latency_single)
  )
)

singles <- tibble(name = names(single_ops), value = single_ops) %>%
  dplyr::mutate(
    name = stringr::str_match(name, "(.*?)_single")[, 2]
  ) %>%
  filter(!is.na(name))

# single_selected <- tar_select_targets(single_ops, contains("_single"))
#
# singles <- tibble(
#   name = names(single_selected),
#   value = single_selected[names(single_selected)]
# ) %>%
#   dplyr::mutate(
#     new_name = stringr::str_match(name, "(.*?)_single")[, 2]
#   )

assertthat::assert_that(nrow(singles) > 0)

combined_single_data <-
  tar_eval(
    tar_combine_raw(
      name,
      value,
      command = expression(bind_rows(!!!.x))
    ),
    values = singles
  )

combined_single_data_processed <- list(
  tar_target(
    functions_total,
    command = load_functions_total(functions)
  )
)

combined_graphs <-
  list(
    tar_target(
      name = provisioned_simple_graph,
      command = output_provisioned_simple(
        functions_total,
        node_levels
      )
    )
  )

graphs <- tibble(name = names(single_ops), value = single_ops) %>%
  dplyr::mutate(
    graph_name = stringr::str_match(name, "(.*?)_graph")[, 2]
  ) %>%
  filter(!is.na(graph_name))

assertthat::assert_that(nrow(graphs) > 0)

combined_plots <-
  tar_eval(
    tar_combine_raw(
      name,
      value,
      command = expression(write_multigraphs(name, !!!.x))
    ),
    values = graphs
  )

graphs2_in <- tar_select_targets(combined_graphs, contains("graph"))
graphs2 <-
  tibble(
    name = names(graphs2_in),
    value = graphs2_in[names(graphs2_in)]
  ) %>%
  dplyr::mutate(
    name = stringr::str_match(name, "(.*?)_graph")[, 2]
  )

assertthat::assert_that(nrow(graphs2) > 0)

combined_plots2 <-
  tar_eval(
    tar_combine_raw(
      name,
      value,
      command = expression(write_multigraphs(name, !!!.x))
    ),
    values = graphs2
  )

list(
  single_ops,
  combined_single_data,
  combined_single_data_processed,
  combined_graphs,
  combined_plots,
  combined_plots2
)
