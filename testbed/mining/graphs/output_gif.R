output_gif <- function(raw.cpu.observed_from_fog_node, bids_won_function) {
  data <- latency %>%
    filter(field == "raw") %>%
    adjust_timestamps() %>%
    ungroup() %>%
    rename(source = instance, destination = destination_name, latency_value = value) %>%
    select(timestamp, source, destination, folder, latency_value, diff) %>%
    smooth_timestamps() %>%
    group_by(timestamp_group, source, destination, folder) %>%
    summarise(latency_value = mean(latency_value), diff = mean(diff)) %>%
    rename(timestamp = timestamp_group) %>%
    ungroup() %>%
    {
      .
    }

  nodes <- load_csv("provisioned_functions.csv") %>%
    prepare() %>%
    adjust_timestamps() %>%
    group_by(timestamp, folder, instance) %>%
    mutate(provisioned = ifelse(value == 0, -1, value)) %>%
    summarise(provisioned = sum(provisioned), value = sum(value)) %>%
    group_by(folder, instance) %>%
    arrange(timestamp, .by_group = TRUE) %>%
    mutate(provisioned = lag(cumsum(provisioned), default = 0), total_provisioned = lag(cumsum(value), default = 0)) %>%
    rename(source = instance) %>%
    select(source, timestamp, folder, provisioned, total_provisioned) %>%
    smooth_timestamps() %>%
    group_by(timestamp_group, folder, source) %>%
    summarise(provisioned = last(provisioned), total_provisioned = last(total_provisioned)) %>%
    rename(timestamp = timestamp_group) %>%
    ungroup() %>%
    {
      .
    }

  cpu <- raw.cpu.observed_from_fog_node %>%
    adjust_timestamps() %>%
    rename(source = instance) %>%
    select(timestamp, source, folder, usage) %>%
    smooth_timestamps() %>%
    group_by(timestamp_group, folder, source) %>%
    summarise(usage = mean(usage)) %>%
    rename(timestamp = timestamp_group) %>%
    ungroup()

  gif.apdex.raw <- load_csv("proxy.csv") %>%
    prepare() %>%
    adjust_timestamps() %>%
    adjust_timestamps(var_name = "value_raw") %>%
    mutate(value = value_raw) %>%
    rename(function_name = tags) %>%
    extract_function_name_info() %>%
    smooth_timestamps() %>%
    inner_join(bids_won_function %>% select(sla_id, function_name, metric_group, metric_group_group, folder, winner), by = c("sla_id", "function_name", "folder", "metric_group_group", "metric_group")) %>%
    rename(measured_latency = value, source = winner) %>%
    group_by(timestamp_group, sla_id, folder, source) %>%
    summarise(satisfied_count = sum(measured_latency <= latency), total = n()) %>%
    mutate(apdex = satisfied_count / total)

  gif.apdex.by_node <- gif.apdex.raw %>%
    group_by(timestamp_group, folder, source) %>%
    summarise(apdex = mean(apdex)) %>%
    rename(timestamp = timestamp_group) %>%
    ungroup()

  all_combinations <- data %>%
    select(folder, source, destination) %>%
    distinct() %>%
    full_join(
      data %>%
        select(timestamp) %>%
        distinct() %>%
        full_join(nodes %>%
          select(timestamp) %>%
          distinct(), by = "timestamp") %>%
        full_join(gif.apdex.by_node %>%
          select(timestamp) %>%
          distinct(), by = "timestamp") %>%
        full_join(cpu %>%
          select(timestamp) %>%
          distinct(), by = "timestamp"),
      by = character()
    )

  data <- all_combinations %>%
    full_join(data, by = c("timestamp", "folder", "source", "destination")) %>%
    full_join(nodes, by = c("source", "timestamp", "folder")) %>%
    full_join(gif.apdex.by_node, by = c("source", "timestamp", "folder")) %>%
    full_join(cpu, by = c("source", "timestamp", "folder")) %>%
    group_by(folder, source, destination) %>%
    arrange(timestamp, .by_group = TRUE) %>%
    fill(diff, provisioned, total_provisioned) %>%
    ungroup() %>%
    {
      .
    }

  globally_provisioned <- data %>%
    select(source, folder, timestamp, total_provisioned) %>%
    distinct() %>%
    group_by(folder, timestamp) %>%
    summarise(globally_provisioned = sum(total_provisioned)) %>%
    ungroup()

  data <- data %>%
    filter(source != destination) %>%
    select(source, destination, everything()) %>%
    rename(from = source, to = destination)

  data_grouped <- data %>%
    group_by(folder) %>%
    group_split()

  animations <- lapply(data_grouped, FUN = create_plot)
  animations <- foreach(data = data_grouped, .verbose = FALSE, .combine = bind_rows) %dopar% {
    create_plot(data)
  }
  return(animations)
}