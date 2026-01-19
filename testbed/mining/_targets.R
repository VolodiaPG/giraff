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

# Source the configuration and utility files
source("config.R")
source("utils.R")
source("loaders.R")
source("commongraphs.R")
files <- c(
  list.files(file.path("graphs/"), pattern = "\\.R$", full.names = TRUE),
  list.files(file.path("graphs_big/"), pattern = "\\.R$", full.names = TRUE),
  list.files(file.path("loaders/"), pattern = "\\.R$", full.names = TRUE),
  list.files(file.path("text/"), pattern = "\\.R$", full.names = TRUE)
)
for (file in files) {
  source(file)
}

Log("Starting targets")

core_pkgs <- c(
  "tibble",
  "dplyr",
  "purrr",
  "stringr",
  "rlang",
  "formattable",
  "tidyverse"
)

load_pkgs <- c(
  core_pkgs,
  "ggplot2",
  "vroom",
  "zoo",
  "snakecase",
  "foreach",
  "doParallel",
  "multidplyr",
  "jsonlite",
  "future",
  "rlang",
  "archive",
  "formattable"
)

graph_pkgs <- c(
  core_pkgs,
  "ggplot2",
  "cowplot",
  "scales",
  "ggprism",
  "multcompView",
  "jsonlite",
  "igraph",
  "viridis",
  "patchwork",
  "ggbeeswarm",
  "rstatix",
  "ggpubr",
  "forcats",
  "akima"
)

graph_html_pkgs <- c(
  "plotly",
  "htmlwidgets",
  "htmltools"
)

latex_pkgs <- c(
  "tikzDevice",
  "snakecase",
  graph_pkgs
)

# Set default target options with minimal packages
tar_option_set(
  packages = core_pkgs, # Packages that your targets need for their tasks.
  format = "qs", # Optionally set the default storage format. qs is fast.
  controller = crew::crew_controller_local(workers = workers), # Use workers from config.R
  memory = "transient",
  garbage_collection = 1,
  deployment = "worker"
)

cue_loaders <- tar_cue(mode = "never")

single_ops <- tar_map(
  # unlist = FALSE, # Return a nested list from tar_map()
  values = METRICS_ARKS_DF, # the value is named "ark"
  names = name,
  tar_target(
    name = node_levels_single,
    command = load_node_levels(ark),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = nb_nodes_single,
    command = load_nb_nodes(node_levels_single),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = bids_raw_single,
    command = load_bids_raw(ark),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  # tar_target(
  #   name = provisioned_sla_single,
  #   command = load_provisioned_sla(ark),
  #   packages = load_pkgs
  # ),
  tar_target(
    name = functions_single,
    command = load_functions(ark),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = raw_deployment_times_single,
    command = load_raw_deployment_times(ark),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = raw_latency_single,
    command = load_raw_latency(ark),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = otel_single,
    command = load_otel(ark),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = otel_logs_single,
    command = load_otel_logs(ark),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = otel_processed_single,
    command = load_otel_processed(otel_single),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = otel_duration_single,
    command = load_otel_duration(otel_processed_single),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  # tar_target(
  #   name = bids_won_function_single,
  #   command = load_bids_won_function(bids_raw_single, provisioned_sla_single),
  #   packages = load_pkgs
  # ),
  tar_target(
    name = otel_profit_single,
    command = load_profit(otel_processed_single),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = flame_func_single,
    command = flame_functions(
      otel_processed_single
    ),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = otel_degrades_single,
    command = load_otel_degrades(
      otel_logs_single
    ),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = otel_errors_single,
    command = load_otel_errors(
      otel_logs_single
    ),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = flame_with_latency_single,
    command = flame_func_with_latency(
      otel_processed_single,
      raw_latency_single
    ),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = nb_functions_single,
    command = load_nb_functions(
      otel_processed_single,
      nb_nodes_single
    ),
    packages = load_pkgs,
    cue = cue_loaders
  ),
  tar_target(
    name = nb_requests_single,
    command = load_nb_requests(
      otel_processed_single,
      otel_errors_single
    ),
    packages = load_pkgs,
    cue = cue_loaders
  )
)
if (!ONLY_BIG_GRAPHS) {
  single_ops <- list(
    single_ops,
    tar_target(
      name = output_mean_time_to_deploy_simple_graph,
      command = wrap_graph(output_mean_time_to_deploy_simple(
        raw_deployment_times_single
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_loss_graph,
      command = wrap_graph(output_loss(raw_latency_single)),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_raw_latency_graph,
      command = wrap_graph(output_raw_latency(raw_latency_single)),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_otel_graph,
      command = wrap_graph(output_otel_plot(otel_single)),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_otel_creation_duration_graph,
      command = wrap_graph(output_otel_creation_duration(otel_single)),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_otel_budget_graph,
      command = wrap_graph(output_otel_budget_plot(otel_single)),
      packages = graph_pkgs
    ),
    tar_target(
      name = otel_function_latency_graph,
      command = wrap_graph(output_otel_function_latency_plot(
        flame_with_latency_single
      )),
      packages = graph_pkgs
    ),
    # tar_target(
    #   name = output_otel_sla_duration_graph,
    #   command = wrap_graph(output_otel_sla_duration_plot(otel_single)),
    #   packages = graph_pkgs
    # ),
    tar_target(
      name = output_otel_duration_latency_graph,
      command = wrap_graph(output_otel_duration_latency_plot(
        otel_processed_single,
        raw_latency_single
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_otel_profit_graph,
      command = wrap_graph(output_otel_profit_plot(
        otel_processed_single
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_otel_functions_graph,
      command = wrap_graph(output_otel_functions_plot(
        otel_processed_single,
        flame_func_single,
        otel_errors_single
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_otel_budget_per_function_graph,
      command = wrap_graph(output_otel_budget_per_function_plot(
        otel_processed_single
      )),
      packages = graph_pkgs
    )
  )
}

singles <- tibble(name = names(single_ops), value = single_ops) %>%
  dplyr::mutate(
    name = stringr::str_match(name, "(.*?)_single")[, 2]
  ) %>%
  filter(!is.na(name))

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
  ),
  tar_target(
    fallbacks_processed,
    command = load_fallbacks_processed(otel_processed, otel_errors, nb_nodes)
  )
)

combined_graphs <-
  list(
    tar_target(
      name = provisioned_simple_graph,
      command = wrap_graph(output_provisioned_simple(
        functions_total,
        node_levels
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_otel_budget_graph,
      command = wrap_graph(
        big_output_otel_budget_plot(
          otel_profit,
          nb_requests,
          nb_nodes,
          nb_functions
        )
      ),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_otel_fallbacks_graph,
      command = wrap_graph(big_output_otel_fallbacks_plot(
        fallbacks_processed
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_pressure_fallbacks_graph,
      command = wrap_graph(big_pressure_fallbacks_plot(
        otel_processed,
        otel_errors,
        nb_nodes,
        nb_requests
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_otel_nb_requests_graph,
      command = wrap_graph(big_output_otel_nb_requests_plot(
        nb_requests,
        nb_nodes
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_pressure_graph,
      command = wrap_graph(big_output_pressure_plot(
        nb_requests,
        nb_nodes
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_nb_functions_graph,
      command = wrap_graph(big_output_nb_functions_plot(
        nb_functions,
        nb_nodes
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_otel_nb_functions_graph,
      command = wrap_graph(big_output_otel_nb_functions_plot(
        functions_total,
        nb_nodes
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_typical_latencies_graph,
      command = wrap_graph(big_output_typical_latencies_plot(
        flame_with_latency
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_typical_node_latencies_graph,
      command = wrap_graph(big_output_typical_node_latencies_plot(
        raw_latency,
        node_levels
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_nb_nodes_graph,
      command = wrap_graph(big_output_nb_nodes_plot(
        node_levels
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = output_requests_profit_graph,
      command = wrap_graph(output_requests_profit_plot(
        otel_profit,
        nb_requests,
        nb_nodes,
        nb_functions
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_nb_requests_env_live_graph,
      command = wrap_graph(big_output_nb_requests_env_live_plot(
        otel_profit,
        nb_requests,
        nb_nodes
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_nb_success_vs_requests_graph,
      command = wrap_graph(big_output_nb_success_vs_requests_plot(
        nb_functions,
        nb_requests,
        nb_nodes
      )),
      packages = graph_pkgs
    ),
    tar_target(
      name = big_output_nb_success_vs_nb_functions_graph,
      command = wrap_graph(big_output_nb_success_vs_nb_functions_plot(
        nb_functions,
        nb_requests,
        nb_nodes
      )),
      packages = graph_pkgs
    ),
    # Text outputs
    tar_target(
      name = text_output_nb_functions,
      command = text_workload_characteristics(
        nb_functions,
        nb_requests,
        otel_duration,
        nb_nodes
      ),
      packages = core_pkgs
    ),
    tar_target(
      name = text_profit,
      command = text_profit_output(
        otel_profit
      ),
      packages = core_pkgs
    ),
    tar_target(
      name = text_pressure,
      command = text_pressure_output(
        nb_requests,
        otel_duration
      ),
      packages = core_pkgs
    )
  )


if (!requireNamespace("tikzDevice", quietly = TRUE)) {
  latex_exports <- list()
  Log("tikzDevice not found, not compiling graphs")
} else {
  Log("tikzDevice found, compiling graphs")
  latex_exports <- list(
    tar_target(
      name = big_output_typical_latencies_latex,
      command = export_graph_tikz(
        big_output_typical_latencies_graph,
        GRAPH_TWO_COLUMN_WIDTH,
        GRAPH_ONE_COLUMN_HEIGHT
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_output_typical_node_latencies_latex,
      command = export_graph_tikz(
        big_output_typical_node_latencies_graph,
        GRAPH_TWO_COLUMN_WIDTH / 3,
        GRAPH_ONE_COLUMN_HEIGHT
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_output_nb_nodes_latex,
      command = export_graph_tikz(
        big_output_nb_nodes_graph,
        GRAPH_TWO_COLUMN_WIDTH / 3,
        GRAPH_ONE_COLUMN_HEIGHT
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_output_nb_functions_latex,
      command = export_graph_tikz(
        big_output_nb_functions_graph,
        GRAPH_TWO_COLUMN_WIDTH / 3,
        GRAPH_ONE_COLUMN_HEIGHT
      ),
      packages = latex_pkgs
    ),
    # tar_target(
    #   name = big_otel_fallbacks_latex,
    #   command = export_graph_tikz(
    #     big_otel_fallbacks_graph,
    #     GRAPH_TWO_COLUMN_WIDTH,
    #     GRAPH_ONE_COLUMN_HEIGHT,
    #     legend_position = "none"
    #   ),
    #   packages = latex_pkgs
    # ),
    tar_target(
      name = big_otel_fallbacks_latex,
      command = export_graph_tikz(
        big_otel_fallbacks_graph,
        GRAPH_TWO_COLUMN_WIDTH,
        GRAPH_ONE_COLUMN_HEIGHT,
        extract_legend = TRUE,
        legend_width = GRAPH_TWO_COLUMN_WIDTH,
        legend_height = GRAPH_ONE_COLUMN_HEIGHT / 4,
        legend_nrow = 1
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_otel_nb_functions_latex,
      command = export_graph_tikz(
        big_output_otel_nb_functions_graph,
        GRAPH_TWO_COLUMN_WIDTH * 2 / 3,
        GRAPH_ONE_COLUMN_HEIGHT,
        extract_legend = TRUE,
        legend_width = GRAPH_TWO_COLUMN_WIDTH,
        legend_height = GRAPH_ONE_COLUMN_HEIGHT / 4,
        legend_nrow = 1
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_otel_nb_requests_latex,
      command = export_graph_tikz(
        big_otel_nb_requests_graph,
        GRAPH_TWO_COLUMN_WIDTH / 3,
        GRAPH_ONE_COLUMN_HEIGHT,
        legend_position = "none"
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_pressure_latex,
      command = export_graph_tikz(
        big_pressure_graph,
        GRAPH_TWO_COLUMN_WIDTH * 2 / 3,
        GRAPH_ONE_COLUMN_HEIGHT
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_requests_profit_latex,
      command = export_graph_tikz(
        output_requests_profit_graph,
        GRAPH_TWO_COLUMN_WIDTH / 2,
        GRAPH_ONE_COLUMN_HEIGHT,
        legend_position = "none"
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_output_nb_requests_env_live_latex,
      command = export_graph_tikz(
        big_output_nb_requests_env_live_graph,
        GRAPH_TWO_COLUMN_WIDTH / 2,
        GRAPH_ONE_COLUMN_HEIGHT,
        legend_position = "none"
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_output_nb_success_vs_requests_latex,
      command = export_graph_tikz(
        big_output_nb_success_vs_requests_graph,
        GRAPH_TWO_COLUMN_WIDTH,
        GRAPH_TWO_COLUMN_HEIGHT,
        legend_position = "right"
      ),
      packages = latex_pkgs
    ),
    tar_target(
      name = big_output_nb_requests_vs_nb_functions_latex,
      command = export_graph_tikz(
        big_output_nb_success_vs_nb_functions_graph,
        GRAPH_TWO_COLUMN_WIDTH,
        GRAPH_TWO_COLUMN_HEIGHT,
        legend_position = "right"
      ),
      packages = latex_pkgs
    )
  )
}

graphs <- tibble(name = names(single_ops), value = single_ops) %>%
  dplyr::mutate(
    graph_name = stringr::str_match(name, "(.*?)_graph")[, 2]
  ) %>%
  filter(!is.na(graph_name))

# assertthat::assert_that(nrow(graphs) > 0)

combined_plots <-
  tar_eval(
    tar_combine_raw(
      name,
      value,
      command = expression(write_multigraphs(name, !!!.x)),
      packages = graph_html_pkgs
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
      command = expression(write_multigraphs(name, !!!.x)),
      packages = graph_html_pkgs
    ),
    values = graphs2
  )


list(
  single_ops,
  combined_single_data,
  combined_single_data_processed,
  combined_graphs,
  combined_plots,
  combined_plots2,
  latex_exports
)
