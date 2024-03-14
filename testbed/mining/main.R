init <- function() {
  Sys.setenv(VROOM_TEMP_PATH = "./vroom")
  system("mkdir -p ./vroom")
  system("rm ./vroom/* || true")

  # To call python from R
  library(archive)
  library(dplyr)
  library(reticulate)
  library(tidyverse)
  library(igraph)
  library(r2r)
  library(formattable)
  library(stringr)
  library(viridis)
  # library(geomtextpath)
  library(cowplot)
  library(scales)
  library(vroom)
  library(zoo)
  library(ggdist)
  library(gghighlight)
  library(ggrepel)
  library(ggbreak)
  library(grid)
  library(lemon)
  library(ggprism)
  library(ggh4x)
  library(ggExtra)
  library(tibbletime)
  library(snakecase)
  library(foreach)
  library(doParallel)
  library(ggside)
  library(ggbeeswarm)
  library(multidplyr)
  library(ggpubr)
  library(Hmisc)
  library(rstatix)
  library(multcompView)
  library(gganimate)

  library(intergraph)
  library(network)
  library(ggnetwork)
  library(treemapify)
  library(networkD3)
  library(plotly)
  library(htmlwidgets)

  library(memoise)

  library(purrr)
  library(future.apply)
  future::plan("multicore", workers = workers)

  ggplot2::theme_set(theme_prism())
}


source("config.R")
suppressMessages(init())
source("utils.R")

cd <- cachem::cache_disk(rappdirs::user_cache_dir("R-giraff"), max_size = 5 * 1024^3)

if (cd$exists("metrics")) {
  cached <- cd$get("metrics")
  if (!identical(METRICS_ARKS, cached)) {
    cd$reset()
  }
} else {
  cd$reset()
}

if (no_memoization) {
  cd$reset()
}

cd$set("metrics", METRICS_ARKS)

memoised <- function(f) {
  memoise(f, cache = cd)
}

source("loaders.R")
source("graphs.R")

node_levels <- load_node_levels()
provisioned_sla <- load_provisioned_sla()
respected_sla <- load_respected_sla()
bids_raw <- load_bids_raw()
bids_won_function <- load_bids_won_function(bids_raw, provisioned_sla)

export_graph_non_ggplot("sla", output_sla_plot(respected_sla, bids_won_function, node_levels))
export_graph_non_ggplot("respected_sla", output_respected_sla_plot(respected_sla, bids_won_function, node_levels))

export_graph("duration_distribution", output_duration_distribution_plot(provisioned_sla))
export_graph("latency_distribution", output_latency_distribution_plot(provisioned_sla))
export_graph("request_distribution", output_request_distribution(respected_sla))
export_graph("latency_vs_expected_latency", output_latency_vs_expected_latency_plot(respected_sla, bids_won_function))
export_graph("in_flight_time", output_in_flight_time_plot_simple(respected_sla, bids_won_function, node_levels))
export_graph("ran_for", output_ran_for_plot_simple(respected_sla))
export_graph("output_arrival", output_arrival(respected_sla))

node_connections <- load_node_connections()
latency <- load_latency(node_connections)
export_graph("output_latency", output_latency(latency))

raw.cpu.observed_from_fog_node <- load_raw_cpu_observed_from_fog_node()
if (generate_gif) {
  output_gif(raw.cpu.observed_from_fog_node, bids_won_function)
}


earnings_jains_plot_data <- load_earnings_jains_plot_data(node_levels, bids_won_function)
# ggsave("jains.png", output_jains(earnings_jains_plot_data))



functions <- load_functions()
functions_total <- load_functions_total(functions)


# plots.nb_deployed.data <- load_nb_deployed_plot_data(respected_sla, functions_total, node_levels)
# # ggsave("anova_nb_deployed.png", output_anova_nb_deployed(plots.nb_deployed.data))

# plots.respected_sla <- load_respected_sla_plot_data(respected_sla)
# # ggsave("respected_sla.png", output_respected_data_plot(plots.respected_sla))

# ggsave("jains.png", output_jains_index_plot(earnings_jains_plot_data))
raw_deployment_times <- load_raw_deployment_times()
# ggsave("mean_time_to_deploy.png", output_mean_time_to_deploy(raw_deployment_times))
export_graph("mean_time_to_deploy_simple", output_mean_time_to_deploy_simple(raw_deployment_times))
# spending_plot_data <- load_spending_plot_data(bids_won_function)
# ggsave("spending.png", output_spending_plot(spending_plot_data))
export_graph("spending_simple", output_spending_plot_simple(bids_won_function))
# options(width = 1000)
# toto <- load_csv("proxy.csv") %>%
#     # rename(function_name = tags) %>%
#     # extract_function_name_info() %>%
#     filter(req_id == "063ea3fa-b428-4977-a1e4-7588c326b8a4") %>%
#     {
#         .
#     }

# print(toto)
