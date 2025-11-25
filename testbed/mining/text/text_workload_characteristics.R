text_workload_characteristics <- function(
  nb_functions,
  nb_requests,
  durations,
  nb_nodes
) {
  functions <- nb_functions %>%
    ungroup() %>%
    summarise(
      avg = mean(nb_functions),
      max = max(nb_functions),
      p99 = quantile(nb_functions, 0.99)
    )

  write(round(functions %>% pull(avg), 1), file = "figures/avg_functions.txt")
  write(functions %>% pull(max), file = "figures/max_functions.txt")
  write(functions %>% pull(p99), file = "figures/p99_functions.txt")

  total_functions <- nb_functions %>%
    group_by(folder) %>%
    summarise(total = sum(nb_functions)) %>%
    ungroup() %>%
    summarise(min = min(total), max = max(total))

  write(total_functions %>% pull(min), file = "figures/min_total_functions.txt")
  write(total_functions %>% pull(max), file = "figures/max_total_functions.txt")

  total_functions <- nb_functions %>%
    group_by(folder, run, env) %>%
    summarise(total = sum(nb_functions)) %>%
    group_by(run, env) %>%
    summarise(min = min(total), max = max(total)) %>%
    mutate(span = max / min) %>%
    ungroup() %>%
    summarise(span = max(span))

  write(
    total_functions %>% pull(span) %>% round(1),
    file = "figures/span_total_functions.txt"
  )

  total_requests <- nb_requests %>%
    group_by(folder) %>%
    summarise(total = sum(requests)) %>%
    ungroup() %>%
    summarise(min = min(total), max = max(total))

  write(total_requests %>% pull(min), file = "figures/min_total_requests.txt")
  write(total_requests %>% pull(max), file = "figures/max_total_requests.txt")

  total_requests <- nb_requests %>%
    left_join(durations) %>%
    filter(duration != 0) %>%
    mutate(throughput = requests / as.numeric(duration)) %>%
    ungroup() %>%
    summarise(min = min(throughput), max = max(throughput))

  # Log(
  #   durations %>%
  #     select(duration) %>%
  #     mutate(duration = duration / 60) %>%
  #     arrange(desc(duration))
  # )

  write(
    total_requests %>% pull(min) %>% round(1),
    file = "figures/min_throughput.txt"
  )
  write(
    total_requests %>% pull(max) %>% round(1),
    file = "figures/max_throughput.txt"
  )

  successes <- nb_requests %>%
    mutate(successes = success / requests * 100) %>%
    ungroup() %>%
    summarize(
      mean_success = mean(successes),
      n = n(),
      std_dev = sd(successes),
      margin_error = qt(0.975, df = n - 1) * std_dev / sqrt(n),
      lower_ci = mean_success - margin_error,
      upper_ci = mean_success + margin_error
    )

  mean_success <- successes %>% pull(mean_success) %>% round(1)
  success_margin_error <- successes %>% pull(margin_error) %>% round(1)

  write(
    paste0(mean_success, "\\% \\pm ", success_margin_error, "\\%"),
    file = "figures/mean_success.txt"
  )

  max_nb_nodes <- nb_nodes %>%
    ungroup() %>%
    summarise(max = max(nb_nodes), min = min(nb_nodes))

  write(
    max_nb_nodes %>% pull(max),
    file = "figures/max_nb_nodes_app.txt"
  )

  write(
    max_nb_nodes %>% pull(min),
    file = "figures/min_nb_nodes_app.txt"
  )

  apps <- nb_functions %>%
    ungroup() %>%
    select(folder, service.namespace) %>%
    distinct() %>%
    group_by(folder) %>%
    summarise(total = n()) %>%
    ungroup() %>%
    summarise(min = min(total), max = max(total))

  write(apps %>% pull(min), file = "figures/min_apps.txt")
  write(apps %>% pull(max), file = "figures/max_apps.txt")
}
