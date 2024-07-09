load_names_raw <- function(ark) {
  names_raw <- load_single_csv(ark, "names.csv") %>%
    rename(instance_address = instance, instance = name) %>%
    select(instance, instance_address, folder) %>%
    distinct()

  return(names_raw)
}

load_raw_latency <- function(ark) {
  return(load_single_csv(ark, "neighbor_latency.csv") %>%
    prepare() %>%
    prepare_convert() %>%
    inner_join(load_names_raw(ark) %>% rename(instance_to = instance_address, destination_name = instance), c("instance_to", "folder")) %>%
    mutate(destination_name = to_snake_case(destination_name)))
}

load_node_connections <- function(ark) {
  node_connections <- load_single_csv(ark, "network_shape.csv") %>%
    mutate(latency = as.numeric(latency)) %>%
    mutate(source = to_snake_case(source), destination = to_snake_case(destination))

  return(node_connections)
}

load_node_levels <- function(ark) {
  node_levels <- load_single_csv(ark, "node_levels.csv") %>%
    rename(name = source, level_value = level) %>%
    mutate(
      name = to_snake_case(name),
      level = case_when(
        level_value == 0 ~ "Cloud",
        level_value == max(level_value) - 1 ~ "Edge+1",
        level_value == max(level_value) ~ "Edge",
        TRUE ~ paste("Cloud", as.character(level_value), sep = "+")
      )
    )

  return(node_levels)
}

load_latency <- function(ark, node_connections) {
  node_connections_renamed <- node_connections %>%
    rename(instance = source, destination_name = destination, goal = latency)

  latency <- load_raw_latency(ark) %>%
    select(destination_name, field, value, instance, timestamp, folder, metric_group, metric_group_group) %>%
    inner_join(node_connections_renamed %>%
      full_join(node_connections_renamed %>%
        mutate(toto = instance, instance = destination_name, destination_name = toto))) %>%
    mutate(diff = value - goal)
  return(latency)
}

load_functions <- function(ark) {
  refused <- tryCatch(
    {
      load_single_csv(ark, "refused_function_gauge.csv") %>%
        prepare() %>%
        prepare_convert() %>%
        extract_function_name_info() %>%
        select(instance, sla_id, folder, metric_group, metric_group_group, docker_fn_name) %>%
        distinct() %>%
        mutate(status = "refused") %>%
        group_by(instance, folder, docker_fn_name, metric_group, metric_group_group, status) %>%
        summarise(n = n())
    },
    error = function(cond) {
      df <- data.frame(instance = character(0), folder = character(0), metric_group = character(0), metric_group_group = character(0), docker_fn_name = character(0), status = character(0), n = numeric(0))
      Log(paste0("cannot get refused gauge functions for ", ark))
      return(df)
    }
  )

  failed <- tryCatch(
    {
      load_single_csv(ark, "send_fails.csv") %>%
        prepare() %>%
        prepare_convert() %>%
        rename(function_name = tag) %>%
        extract_function_name_info() %>%
        select(instance, folder, metric_group, metric_group_group, docker_fn_name) %>%
        distinct() %>%
        mutate(status = "failed") %>%
        group_by(instance, folder, metric_group, metric_group_group, status, docker_fn_name) %>%
        summarise(n = n())
    },
    error = function(cond) {
      df <- data.frame(instance = character(0), docker_fn_name = character(0), folder = character(0), metric_group = character(0), metric_group_group = character(0), status = character(0), n = numeric(0))
      Log(paste0("cannot get failed gauge functions for ", ark))
      return(df)
    }
  )

  functions <- load_single_csv(ark, "provisioned_function_gauge.csv") %>%
    prepare() %>%
    prepare_convert() %>%
    extract_function_name_info() %>%
    select(instance, sla_id, folder, metric_group, metric_group_group, docker_fn_name) %>%
    distinct() %>%
    mutate(status = "provisioned") %>%
    group_by(instance, folder, metric_group, metric_group_group, status, docker_fn_name) %>%
    summarise(n = n()) %>%
    full_join(refused) %>%
    full_join(failed)

  return(functions)
}

load_functions_all_total <- function(functions) {
  total <- functions %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group, metric_group_group) %>%
    select(-status) %>%
    summarise(provisioned = sum(n))

  total <- functions %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(total = sum(n)) %>%
    inner_join(total, by = c("folder", "metric_group_group", "metric_group")) %>%
    mutate(ratio = provisioned / total)

  return(total)
}

load_functions_total <- function(functions) {
  total <- functions %>%
    group_by(folder, instance, metric_group, metric_group_group, docker_fn_name) %>%
    summarise(total = sum(n))

  functions_total <- functions %>%
    inner_join(total, by = c("instance", "folder", "metric_group", "metric_group_group", "docker_fn_name")) %>%
    group_by(folder, status, metric_group, docker_fn_name, metric_group_group, total) %>%
    summarise(n = sum(n)) %>%
    mutate(ratio = n / total)

  return(functions_total)
}

load_bids_raw <- function(ark) {
  bids_raw <- load_single_csv(ark, "bid_gauge.csv") %>%
    prepare() %>%
    prepare_convert()

  if ((bids_raw %>% ungroup() %>% filter(field == "price") %>% summarise(n = n()))$n == 0) {
    stop(paste0("Error, there are no price/bid cols in ", bids_raw %>% ungroup() %>% select(folder) %>% distinct()))
  }

  bids_raw <- bids_raw %>%
    # group_by(folder, metric_group, metric_group_group) %>%
    select(folder, sla_id, metric_group, metric_group_group, field, bid_id, function_name, value, instance) %>%
    group_by(folder, sla_id, metric_group, metric_group_group, field, bid_id, function_name, instance) %>%
    spread(field, value) %>%
    mutate(bid = ifelse(bid < 0 & bid >= -0.001, 0, bid))

  if ((bids_raw %>% ungroup() %>% filter(bid < 0.0) %>% summarise(n = n()))$n != 0) {
    Log(bids_raw %>% ungroup() %>% filter(bid < 0.0))
    stop(paste0("Error, there are negative bids in ", bids_raw %>% ungroup() %>% filter(bid < 0.0) %>% select(folder) %>% distinct()))
  }

  return(bids_raw)
}

load_provisioned_sla <- function(ark) {
  provisioned_sla <- load_single_csv(ark, "function_deployment_duration.csv") %>%
    prepare() %>%
    prepare_convert() %>%
    select(bid_id, sla_id, folder, metric_group, metric_group_group, function_name) %>%
    distinct() %>%
    extract_function_name_info()

  return(provisioned_sla)
}

load_bids_won_function <- function(bids_raw, provisioned_sla) {
  bids_won_function <- bids_raw %>%
    select(sla_id, bid_id, instance, function_name, folder, metric_group, metric_group_group, price, bid) %>%
    distinct() %>%
    inner_join(provisioned_sla, by = c("bid_id", "sla_id", "folder", "metric_group", "metric_group_group", "function_name")) %>%
    mutate(winner = instance) %>%
    mutate(cost = price) %>%
    select(sla_id, function_name, folder, metric_group, metric_group_group, winner, cost, bid)

  return(bids_won_function)
}

load_raw_cpu_all <- function(ark) {
  cpu <- load_single_csv(ark, "cpu_observed_from_fog_node.csv") %>%
    prepare() %>%
    prepare_convert() %>%
    select(-value) %>%
    spread(field, value_raw)
  return(cpu)
}

load_raw_cpu_observed_from_fog_node <- function(ark) {
  cpu <- load_single_csv(ark, "cpu_observed_from_fog_node.csv") %>%
    prepare() %>%
    prepare_convert()

  return(cpu %>% filter(field == "initial_allocatable") %>%
    rename(initial_allocatable = value) %>%
    inner_join(cpu %>%
      filter(field == "used") %>%
      rename(used = value), by = c("timestamp", "folder", "instance", "metric_group", "metric_group_group")) %>%
    mutate(usage = used / initial_allocatable) %>%
    select(instance, timestamp, usage, folder, metric_group, metric_group_group))
}

load_auc_usage_cpu <- function() {
  registerDoParallel(cl = parallel_loading_datasets_small, cores = parallel_loading_datasets_small)
  raw_auc_usage_cpu <- bind_rows(foreach(ark = METRICS_ARKS) %dopar% {
    load_single_csv(ark, "cpu_observed_from_fog_node.csv") %>%
      prepare() %>%
      prepare_convert() %>%
      get_usage()
  })

  gc()
  return(raw_auc_usage_cpu)
}

## load_auc_usage_mem <- function() {
##    raw_auc_usage_mem <- bind_rows(foreach(ark = METRICS_ARKS) %dopar% {
##        load_single_csv(ark, "memory_observed_from_fog_node.csv") %>%
##            prepare() %>%
##            prepare_convert() %>%
##            get_usage()
##    })
##
##    gc()
##    return(raw_auc_usage_mem)
## })

load_total_gains <- function(bids_won_function) {
  total_gains <- bids_won_function %>%
    group_by(folder, metric_group, metric_group_group, winner) %>%
    summarise(earnings = sum(cost))
  return(total_gains)
}

load_grand_total_gains <- function(bids_won_function) {
  grand_total_gains <- bids_won_function %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(grand_total = sum(cost))
  return(grand_total_gains)
}

## load_errors <- function() {
##    errors <- tryCatch(
##        {
##            load_csv("iot_emulation_http_request_to_processing_echo_fails.csv") %>%
##                prepare() %>%
##                prepare_convert() %>%
##                extract_function_name_info() %>%
##                distinct()
##        },
##        error = function(cond) {
##            columns <- c("instance", "job", "timestamp", "tag", "period", "folder", "metric_group", "latency", "value")
##            df <- data.frame(
##                instance = character(0),
##                job = character(0),
##                period = numeric(0),
##                folder = character(0),
##                metric_group = character(0),
##                latency = character(0),
##                value = numeric(0)
##            )
##            return(df)
##        }
##    )
##    return(errors)
## })
#
load_raw_deployment_times <- function(ark) {
  raw_deployment_times <- load_single_csv(ark, "function_deployment_duration.csv") %>%
    prepare() %>%
    prepare_convert() %>%
    extract_function_name_info()
  return(raw_deployment_times)
}

load_paid_functions <- function(ark) {
  paid_functions <- load_single_csv(ark, "paid_functions.csv") %>%
    prepare() %>%
    prepare_convert() %>%
    extract_function_name_info()
  return(paid_functions)
}

load_earnings_jains_plot_data <- function(node_levels, bids_won_function) {
  earnings <- node_levels %>%
    rename(winner = name) %>%
    full_join(bids_won_function %>% group_by(folder, winner, metric_group_group, metric_group) %>% summarise(earnings = sum(cost))) %>%
    mutate(earnings = ifelse(is.na(earnings), 0, earnings)) %>%
    group_by(metric_group, metric_group_group, folder) %>%
    summarise(jains_index = jains_index(earnings), worst_case = round(1 / n(), 2), n = n()) %>%
    rename(score = jains_index)
  return(earnings)
}


load_respected_sla <- function(ark) {
  acceptable_chain_cumulative <- function(prev, current) {
    return(prev & current)
  }
  cluster <- new_cluster(4)

  cluster_library(cluster, "purrr")
  cluster_library(cluster, "dplyr")
  cluster_library(cluster, "multidplyr")
  cluster_copy(cluster, "acceptable_chain_cumulative")

  respected_sla <- load_single_csv(ark, "proxy.csv") %>%
    prepare() %>%
    group_by(folder) %>%
    adjust_timestamps(var_name = "timestamp", reference = "value_raw") %>%
    adjust_timestamps(var_name = "value_raw", reference = "value_raw") %>%
    mutate(value = value_raw) %>%
    rename(function_name = tags) %>%
    extract_function_name_info() %>%
    group_by(folder, metric_group, metric_group_group, req_id) %>%
    partition(cluster) %>%
    arrange(desc(first_req_id), value) %>%
    mutate(ran_for = timestamp - value) %>%
    mutate(prev = ifelse(first_req_id == TRUE, value, timestamp)) %>%
    mutate(in_flight = value - lag(prev)) %>%
    mutate(total_process_time = first(ran_for)) %>%
    group_by(folder, metric_group, metric_group_group, req_id) %>%
    mutate(prev_function = lag(ifelse(first_req_id, "<iot_emulation>", docker_fn_name))) %>%
    mutate(prev_sla = lag(ifelse(first_req_id, "<iot_emulation>", sla_id))) %>%
    filter(first_req_id == FALSE) %>%
    mutate(acceptable = (service_status == 200) & (in_flight <= latency + 0.001)) %>%
    mutate(acceptable_chained = accumulate(acceptable, `&`)) %>%
    collect() %>%
    group_by(
      sla_id,
      folder,
      metric_group,
      metric_group_group,
      function_name,
      docker_fn_name,
      prev_function,
      prev_sla
    ) %>%
    partition(cluster) %>%
    summarise(
      acceptable = sum(acceptable),
      all_errors = sum((status != 200) & (service_status != 200)),
      acceptable_chained = sum(acceptable_chained),
      total = dplyr::n(),
      measured_latency = mean(in_flight),
      ran_for = max(ran_for),
      service_oked = sum((service_status >= 200) & (service_status < 300)),
      service_timeouted = sum(service_status == 408),
      service_not_found = sum((service_status == 404)),
      service_errored = sum((service_status >= 400) & (service_status < 500)) - service_timeouted - service_not_found,
      service_server_errored = sum(service_status >= 500),
      proxy_oked = sum((status >= 200) & (status < 300)),
      proxy_timeouted = sum(status == 408),
      proxy_not_found = sum((status == 404)),
      proxy_errored = sum((status >= 400) & (status < 500)) - proxy_timeouted - proxy_not_found,
      proxy_server_errored = sum(status >= 500),
    ) %>%
    filter(!is.na(acceptable)) %>%
    filter(!is.na(acceptable_chained)) %>%
    collect()

  # Log(respected_sla %>% group_by(sla_id, folder) %>% filter(is.na(acceptable)) %>% select(sla_id, folder))
  # Log("titi")
  # Log(respected_sla %>% filter(req_id == "3ad1345e-56db-4655-8f09-d70707f08786") %>% select(sla_id, req_id, value, timestamp, acceptable, docker_fn_name, in_flight, first_req_id))
  # Log(respected_sla %>% filter(req_id == "59e081e1-0240-4b4f-a3f4-afaf53e6972f") %>% select(sla_id, req_id, value, timestamp, acceptable, docker_fn_name, in_flight, first_req_id))

  return(respected_sla)
}

load_nb_deployed_plot_data <- function(respected_sla_headers, functions_total, node_levels) {
  plot <- respected_sla_headers %>%
    extract_function_name_info() %>%
    left_join(functions_total %>% select(folder, metric_group, metric_group_group, load_type, latency_type, total) %>% rename(nb_functions_requested_total = total)) %>%
    left_join(node_levels %>% group_by(folder) %>% summarise(nb_nodes = n())) %>%
    # mutate(nb_nodes_group = case_when(
    #     nb_nodes < 19 ~ "Danger zone 1",
    #     nb_nodes <= 34 ~ "$19 \\le n < 34$",
    #     nb_nodes < 112 ~ "Danger zone 2",
    #     nb_nodes <= 119 ~ "$112 \\le n \\le 119$",
    #     TRUE ~ "Danger zone 3",
    # )) %>%
    rename(nb_nodes_group = nb_nodes) %>%
    group_by(folder, metric_group_group, metric_group, nb_nodes_group, nb_functions_requested_total) %>%
    summarise(funcs = n()) %>%
    mutate(nb_functions = funcs / nb_functions_requested_total) %>%
    correct_names()

  return(plot)
}

load_respected_sla_plot_data <- function(respected_sla.header.nb) {
  plot <- respected_sla.header.nb %>%
    extract_function_name_info() %>%
    # left_join(raw.nb_functions.total.full %>% rename(total_func = total)) %>%
    # left_join(raw.nb_functions.total.ll %>% rename(total_func_ll = total)) %>%
    rowwise() %>%
    # mutate(ratio_func_ll = total_func_ll / total_func) %>%
    mutate(satisfied_count = count.acceptable) %>%
    # mutate(satisfied_count = abs(measured_latency - latency) / latency) %>%
    mutate(measured_latency = abs(measured_latency) / latency) %>%
    # mutate(ratio_func_ll = sprintf("%.1f%% low-latency ƒ", ratio_func_ll *100))  %>%
    correct_names() %>%
    mutate(toto = "toto")
  # mutate(group = sprintf("%s\n%s\n%s", latency_type, load_type, ratio_func_ll))
  # mutate(group = latency_type)
  return(plot)
}

load_spending_plot_data <- function(bids_won_function) {
  plot <- bids_won_function %>%
    extract_function_name_info() %>%
    # left_join(raw.nb_functions) %>%
    # mutate(ratio_func_ll = total_func_ll / total_func) %>%
    # mutate(ratio_func_ll = sprintf("%.1f%% ll ƒ", ratio_func_ll *100)) %>%
    group_by(folder, metric_group_group, metric_group) %>%
    summarise(spending = mean(cost)) %>%
    # mutate(fn_category = sprintf("%s\n%s", latency_type, load_type)) %>%
    correct_names()
  return(plot)
}
