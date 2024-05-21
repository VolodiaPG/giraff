init <- function() {
  Sys.setenv(VROOM_TEMP_PATH = "./vroom")
  system("mkdir -p ./vroom")
  system("rm ./vroom/* || true")

  # To call python from R
  library(archive)
  library(dplyr)
  # library(reticulate)
  library(tidyverse)
  library(igraph)
  # library(r2r)
  library(formattable)
  library(stringr)
  library(viridis)
  # library(geomtextpath)
  # library(cowplot)
  library(scales)
  library(vroom)
  library(zoo)
  # library(ggdist)
  # library(gghighlight)
  # library(ggrepel)
  # library(ggbreak)
  # library(grid)
  # library(lemon)
  # library(ggforce)
  library(ggprism)
  # library(ggh4x)
  # library(ggExtra)
  # library(tibbletime)
  library(snakecase)
  library(foreach)
  library(doParallel)
  # library(ggside)
  library(ggbeeswarm)
  library(multidplyr)
  # library(ggpubr)
  # library(Hmisc)
  # library(rstatix)
  # library(multcompView)

  # library(intergraph)
  ## library(treemapify)
  # library(networkD3)
  library(plotly)
  library(htmlwidgets)
  library(htmltools)

  library(memoise)

  library(purrr)
  library(future.apply)
  library(rlang)

  future::plan("multicore", workers = workers)

  ggplot2::theme_set(theme_prism())
}


source("config.R")
suppressMessages(init())

log.socket <- make.socket(port = 4000)

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

mem <- function(cb) {
  memoise(cb, cache = cd)
}

source("utils.R")


graph <- mem(function(name, graphs, graph) {
  df <- data.frame(name = name, type = "ggplot", graph = I(list(graph)))
  return(bind_rows(graphs, df))
})

graph_non_ggplot <- mem(function(name, graphs, graph) {
  df <- data.frame(name = name, type = "noggplot", graph = I(list(graph)))
  return(bind_rows(graphs, df))
})

source("loaders.R")
source("graphs.R")
source("commongraphs.R")
combine <- function(arks, cb) {
  cb_name <- deparse(substitute(cb))
  return(combine2(arks, cb_name, cb))
}

cl <- makeForkCluster(workers)
registerDoParallel(cl = cl)


combine2 <- memoise2(function(arks, cb_name, cb) {
  Log(paste0("Combining ", cb_name))
  tmp <- foreach(ark = arks, .verbose = FALSE, .packages = c("purrr", "dplyr", "multidplyr"), .combine = bind_rows) %dopar% mem(cb)(ark)

  return(tmp)
}, cache = cd, expr_vars = c("cb"))


# functions <- load_functions()
if (single_graphs) {
  Log("Doing single graphs")
  # memoize useful loaders
  m_load_node_levels <- mem(load_node_levels)
  m_load_provisioned_sla <- mem(load_provisioned_sla)
  m_load_respected_sla <- mem(load_respected_sla)
  m_load_bids_raw <- mem(load_bids_raw)
  m_load_bids_won_function <- mem(load_bids_won_function)
  m_load_functions <- mem(load_functions)
  m_load_node_connections <- mem(load_node_connections)
  m_load_latency <- mem(load_latency)
  m_load_raw_latency <- mem(load_raw_latency)
  m_load_raw_deployment_times <- mem(load_raw_deployment_times)

  graphs <- foreach(ark = METRICS_ARKS, .verbose = FALSE, .combine = bind_rows) %dopar% {
    node_levels <- m_load_node_levels(ark)
    provisioned_sla <- m_load_provisioned_sla(ark)
    respected_sla <- m_load_respected_sla(ark)
    bids_raw <- m_load_bids_raw(ark)
    bids_won_function <- m_load_bids_won_function(bids_raw, provisioned_sla)
    functions <- m_load_functions(ark)
    # nb_deployed <- load_nb_deployed_data(respected_sla, functions_total, node_levels)

    node_connections <- m_load_node_connections(ark)
    latency <- m_load_latency(ark, node_connections)
    raw_latency <- m_load_raw_latency(ark)
    raw_deployment_times <- m_load_raw_deployment_times(ark)

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
      Log("Doing GIF")
      library(gganimate)
      library(network)
      library(ggnetwork)

      raw.cpu.observed_from_fog_node <- load_raw_cpu_observed_from_fog_node(ark)
      output_gif(raw.cpu.observed_from_fog_node, bids_won_function)
    }

    return(
      graphs %>%
        mutate(tag = ark)
    )
  }
  write_multigraphs(graphs)
}

Log("Doing combined graphs")
node_levels <- combine(METRICS_ARKS, load_node_levels)
bids_raw <- combine(METRICS_ARKS, load_bids_raw)
provisioned_sla <- combine(METRICS_ARKS, load_provisioned_sla)
functions <- combine(METRICS_ARKS, load_functions)
respected_sla <- combine(METRICS_ARKS, load_respected_sla)
raw_deployment_times <- combine(METRICS_ARKS, load_raw_deployment_times)

Log("Loading additionnal full sets")
functions_total <- mem(load_functions_total)(functions)
functions_all_total <- mem(load_functions_all_total)(functions)
bids_won_function <- mem(load_bids_won_function)(bids_raw, provisioned_sla)
earnings_jains_plot_data <- mem(load_earnings_jains_plot_data)(node_levels, bids_won_function)

export_graph("provisioned", mem(output_provisioned_simple)(functions_total, node_levels))
export_graph("provisioned_total", mem(output_provisioned_simple_total)(functions_total, node_levels))
export_graph("jains", mem(output_jains_simple)(earnings_jains_plot_data, functions_all_total, node_levels))
export_graph("spending_total", mem(output_spending_plot_simple_total)(bids_won_function, node_levels))
export_graph("respected_sla_plot_total", mem(output_respected_data_plot_total)(respected_sla, functions_all_total, node_levels))
export_graph("requests_served", mem(output_number_requests)(respected_sla, node_levels))
export_graph("total_requests_served_total", mem(output_number_requests_total)(respected_sla, node_levels))
export_graph("requests_served_v_provisioned", mem(output_requests_served_v_provisioned)(respected_sla, functions_total, node_levels))
export_graph("mean_time_to_deploy_total", mem(output_mean_time_to_deploy_simple_total)(raw_deployment_times, node_levels))
export_graph("output_non_respected", mem(output_non_respected)(respected_sla, functions_all_total, node_levels))


parallel::stopCluster(cl)
