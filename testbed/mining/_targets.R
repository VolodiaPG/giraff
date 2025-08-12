# Created by use_targets().
# Follow the comments below to fill in this target script.
# Then follow the manual to check and run the pipeline:
#   https://books.ropensci.org/targets/walkthrough.html#inspect-the-pipeline

# Load packages required to define the pipeline:
library(targets)
library(tarchetypes) # Load other packages as needed.
library(crew) # For parallel processing
# library(dplyr)

# Source the configuration and utility files
source("config.R")
source("utils.R")
source("loaders.R")
source("commongraphs.R")
source("graphs.R")

log.socket <- make.socket(port = 4001)

# Set target options:
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
  controller = crew::crew_controller_local(workers = all_workers) # Use workers from config.R
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
    name = output_otel_plot_graph,
    command = output_otel_plot(otel_single)
  ),
  tar_target(
    name = output_otel_budget_plot_graph,
    command = output_otel_budget_plot(otel_single)
  ),
  tar_target(
    name = output_otel_correlations_plot_graph,
    command = output_otel_correlations_plot(otel_single, raw_latency_single)
  )
)

# command = dplyr::bind_rows(!!!.x)
combined_ops <- list(
  # Combined graphs section
  tar_combine(
    node_levels_combined,
    tar_select_targets(single_ops, starts_with("node_levels_single_"))
  ),
  tar_combine(
    bids_raw_combined,
    tar_select_targets(single_ops, starts_with("bids_raw_single_"))
  ),
  tar_combine(
    provisioned_sla_combined,
    tar_select_targets(single_ops, starts_with("provisioned_sla_single_"))
  ),
  tar_combine(
    functions_combined,
    tar_select_targets(single_ops, starts_with("functions_single_"))
  ),
  # tar_combine(
  #   provisioned_functions_combined,
  #   tar_select_targets(
  #     single_ops,
  #     starts_with("provisioned_functions_single_")
  #   )
  # )
  # tar_combine(
  #   respected_sla_combined,
  #   tar_select_targets(single_ops, starts_with("respected_sla_single_"))
  # )
  # tar_combine(
  #   acceptable_sla_combined,
  #   tar_select_targets(
  #     single_ops,
  #     starts_with("single_acceptable_from_respected_slas_")
  #   )
  # )
  tar_combine(
    raw_deployment_times_combined,
    tar_select_targets(
      single_ops,
      starts_with("raw_deployment_times_single_")
    )
  )
  # tar_combine(
  #   paid_functions_combined,
  #   tar_select_targets(single_ops, starts_with("paid_functions_single_"))
  # )
)

# tar_target(
#   name = functions_total_combined,
#   command = load_functions_total(functions_combined)
# ),
# tar_target(
#   name = functions_all_total_combined,
#   command = load_functions_all_total(functions_combined)
# ),
# tar_target(
#   name = bids_won_function_combined,
#   command = load_bids_won_function(
#     bids_raw_combined,
#     provisioned_sla_combined
#   )
# ),
# tar_target(
#   name = earnings_jains_plot_data_combined,
#   command = load_earnings_jains_plot_data(
#     node_levels_combined,
#     bids_won_function_combined
#   )
# ),
#
# # Combined graph targets
# tar_target(
#   name = provisioned_graph,
#   command = output_provisioned_simple(
#     functions_total_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = provisioned_total_graph,
#   command = output_provisioned_simple_total(
#     functions_total_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = jains_graph,
#   command = output_jains_simple(
#     earnings_jains_plot_data_combined,
#     functions_all_total_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = spending_total_graph,
#   command = output_spending_plot_simple_total(
#     bids_won_function_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = respected_sla_plot_total_graph,
#   command = output_respected_data_plot_total(
#     respected_sla_combined,
#     functions_all_total_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = requests_served_graph,
#   command = output_number_requests(
#     respected_sla_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = total_requests_served_total_graph,
#   command = output_number_requests_total(
#     respected_sla_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = requests_served_v_provisioned_graph,
#   command = output_requests_served_v_provisioned(
#     respected_sla_combined,
#     functions_total_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = mean_time_to_deploy_total_graph,
#   command = output_mean_time_to_deploy_simple_total(
#     raw_deployment_times_combined,
#     node_levels_combined,
#     paid_functions_combined
#   )
# ),
# tar_target(
#   name = output_non_respected_graph,
#   command = output_non_respected(
#     respected_sla_combined,
#     functions_all_total_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = output_spider_chart_graph,
#   command = output_placement_method_comparison(
#     respected_sla_combined,
#     functions_total_combined,
#     node_levels_combined,
#     bids_won_function_combined,
#     raw_deployment_times_combined
#   )
# ),
# tar_target(
#   name = output_mean_respected_slas_graph,
#   command = output_mean_respected_slas(
#     acceptable_sla_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = output_mean_deployment_times_graph,
#   command = output_mean_deployment_times(
#     raw_deployment_times_combined,
#     node_levels_combined,
#     respected_sla_combined
#   )
# ),
# tar_target(
#   name = output_mean_spending_graph,
#   command = output_mean_spending(
#     bids_won_function_combined,
#     node_levels_combined,
#     respected_sla_combined
#   )
# ),
# tar_target(
#   name = output_mean_placed_functions_per_node_graph,
#   command = output_mean_placed_functions_per_node(
#     provisioned_functions_combined,
#     node_levels_combined
#   )
# ),
# tar_target(
#   name = output_mean_latency_graph,
#   command = output_mean_latency(respected_sla_combined, node_levels_combined)
# ),
#
# # Collect all combined graphs into a list
# tar_target(
#   name = all_combined_graphs,
#   command = list(
#     provisioned_graph,
#     provisioned_total_graph,
#     jains_graph,
#     spending_total_graph,
#     respected_sla_plot_total_graph,
#     requests_served_graph,
#     total_requests_served_total_graph,
#     requests_served_v_provisioned_graph,
#     mean_time_to_deploy_total_graph,
#     output_non_respected_graph,
#     output_spider_chart_graph,
#     output_mean_respected_slas_graph,
#     output_mean_deployment_times_graph,
#     output_mean_spending_graph,
#     output_mean_placed_functions_per_node_graph,
#     output_mean_latency_graph
#   )
# ),
#
# # Write all combined graphs to files
# tar_target(
#   name = write_combined_graphs,
#   command = write_multigraphs(all_combined_graphs),
#   format = "file"
# )
# )

list(
  single_ops,
  combined_ops
)
