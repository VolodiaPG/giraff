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
  library(htmltools)

  library(memoise)

  library(purrr)
  library(future.apply)
  future::plan("multicore", workers = workers)

  ggplot2::theme_set(theme_prism())
}


source("config.R")
suppressMessages(init())
source("utils.R")

cd <- cachem::cache_disk(rappdirs::user_cache_dir("R-giraff"), max_size = 20 * 1024^3)

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

combine_all_loaded <- memoised(function(arks, cb) {
  registerDoParallel(cl = workers, cores = workers)
  return(bind_rows(foreach(ark = arks) %dopar% {
    return(cb(ark))
  }))
})

graph <- memoised(function(name, graphs, graph) {
  df <- data.frame(name = name, type = "ggplot", graph = I(list(graph)))
  return(bind_rows(graphs, df))
})

graph_non_ggplot <- memoised(function(name, graphs, graph) {
  df <- data.frame(name = name, type = "noggplot", graph = I(list(graph)))
  return(bind_rows(graphs, df))
})

source("loaders.R")
source("graphs.R")
source("commongraphs.R")

# functions <- load_functions()
if (single_graphs) {
  registerDoParallel(cl = workers, cores = workers)
  graphs <- foreach(ark = METRICS_ARKS) %dopar% {
    node_levels <- load_node_levels(ark)
    provisioned_sla <- load_provisioned_sla(ark)
    respected_sla <- load_respected_sla(ark)
    bids_raw <- load_bids_raw(ark)
    bids_won_function <- load_bids_won_function(bids_raw, provisioned_sla)
    functions <- load_functions(ark)
    # nb_deployed <- load_nb_deployed_data(respected_sla, functions_total, node_levels)

    node_connections <- load_node_connections(ark)
    latency <- load_latency(ark, node_connections)
    raw_latency <- load_raw_latency(ark)
    raw_deployment_times <- load_raw_deployment_times(ark)

    graphs <- NULL
    graphs <- graph_non_ggplot("respected_sla", graphs, output_respected_sla_plot(respected_sla, bids_won_function, node_levels))
    graphs <- graph_non_ggplot("sla", graphs, output_sla_plot(respected_sla, bids_won_function, node_levels))

    graphs <- graph("duration_distribution", graphs, output_duration_distribution_plot(provisioned_sla))
    graphs <- graph("latency_distribution", graphs, output_latency_distribution_plot(provisioned_sla))
    graphs <- graph("request_distribution", graphs, output_request_distribution(respected_sla))
    graphs <- graph("latency_vs_expected_latency", graphs, output_latency_vs_expected_latency_plot(respected_sla, bids_won_function))
    graphs <- graph("in_flight_time", graphs, output_in_flight_time_plot_simple(respected_sla, bids_won_function, node_levels))
    graphs <- graph("ran_for", graphs, output_ran_for_plot_simple(respected_sla))
    graphs <- graph("output_arrival", graphs, output_arrival(respected_sla))
    graphs <- graph("output_latency", graphs, output_latency(latency))
    graphs <- graph("output_loss", graphs, output_loss(raw_latency))
    graphs <- graph("spending", graphs, output_spending_plot_simple(bids_won_function, node_levels))

    if (generate_gif) {
      raw.cpu.observed_from_fog_node <- load_raw_cpu_observed_from_fog_node(ark)
      output_gif(raw.cpu.observed_from_fog_node, bids_won_function)
    }

    return(
      graphs %>%
        mutate(tag = ark)
    )
  }
  graphs <- bind_rows(graphs)
  write_multigraphs(graphs)
}

combine <- function(cb) {
  return(combine_all_loaded(METRICS_ARKS, cb))
}
node_levels <- combine(load_node_levels)
bids_raw <- combine(load_bids_raw)
provisioned_sla <- combine(load_provisioned_sla)
functions <- combine(load_functions)
respected_sla <- combine(load_respected_sla)

functions_total <- load_functions_total(functions)
bids_won_function <- load_bids_won_function(bids_raw, provisioned_sla)
earnings_jains_plot_data <- load_earnings_jains_plot_data(node_levels, bids_won_function)

export_graph("provisioned", output_provisioned_simple(functions_total))
export_graph("provisioned_total", output_provisioned_simple_total(functions_total))
export_graph("jains", output_jains_simple(earnings_jains_plot_data))
export_graph("spending_total", output_spending_plot_simple_total(bids_won_function, node_levels))
export_graph("respected_sla_plot", output_respected_data_plot(respected_sla))
export_graph("respected_sla_plot_total", output_respected_data_plot_total(respected_sla))
export_graph("total_requests_served", output_number_requests(respected_sla))
export_graph("total_requests_served_total", output_number_requests_total(respected_sla))


# plots.nb_deployed.data <- load_nb_deployed_plot_data(respected_sla, functions_total, node_levels)
# # ggsave("anova_nb_deployed.png", output_anova_nb_deployed(plots.nb_deployed.data))

# plots.respected_sla <- load_respected_sla_plot_data(respected_sla)
# # ggsave("respected_sla.png", output_respected_data_plot(plots.respected_sla))

# ggsave("jains.png", output_jains_index_plot(earnings_jains_plot_data))
# ggsave("mean_time_to_deploy.png", output_mean_time_to_deploy(raw_deployment_times))
# export_graph("mean_time_to_deploy_simple", output_mean_time_to_deploy_simple(raw_deployment_times))
# spending_plot_data <- load_spending_plot_data(bids_won_function)
# ggsave("spending.png", output_spending_plot(spending_plot_data))
# export_graph("spending_simple", output_spending_plot_simple(bids_won_function))
# options(width = 1000)
# toto <- load_csv("proxy.csv") %>%
#     # rename(function_name = tags) %>%
#     # extract_function_name_info() %>%
#     filter(req_id == "063ea3fa-b428-4977-a1e4-7588c326b8a4") %>%
#     {
#         .
#     }

# print(toto)
