text_pressure_output <- function(nb_requests, durations, fallbacks_processed) {
  df <- nb_requests %>%
    left_join(durations) %>%
    group_by(folder, metric_group, service.namespace) %>%
    filter(duration > 0) %>%
    summarise(
      requests = sum(success) / sum(requests),
      successes = sum(success) / as.numeric(duration),
      .groups = "drop"
    ) %>%
    group_by(folder, metric_group) %>%
    summarise(
      requests = mean(requests),
      successes = mean(successes),
      .groups = "drop"
    ) %>%
    extract_context() %>%
    env_live_extract() %>%
    extract_env_name() %>%
    # filter(env_live %in% c(SCE_ONE, SCE_TWO)) %>%
    group_by(env_live, env) %>%
    summarise(requests = mean(requests), successes = mean(successes)) %>%
    arrange(env_live, env)

  Log(df)

  inc_success <- df[3, ]$requests - df[1, ]$requests
  write(
    paste0(round(inc_success * 100, 1), " percentage points"),
    file = "figures/pressure_increase.txt"
  )

  inc_success <- df[3, ]$requests - df[4, ]$requests
  write(
    paste0(round(inc_success * 100, 1), " percentage points"),
    file = "figures/pressure_increase_btw_loads.txt"
  )

  inc_success <- df[5, ]$requests - df[3, ]$requests
  write(
    paste0(round(inc_success * 100, 1), " percentage points"),
    file = "figures/pressure_increase_ccc.txt"
  )

  # df %>%
  #   mutate(file_name = paste0("pressure_successes_", env_live, ".txt")) %>%
  #   rowwise() %>%
  #   group_walk(
  #     ~ write(
  #       paste0(
  #         round(.x$successes, 1)
  #         # "\\% \\pm ",
  #         # round(.x$margin_error * 100, 1),
  #         # "\\%"
  #       ),
  #       file = paste0("figures/", .x$file_name)
  #     )
  #   )

  p99_df <- nb_requests %>%
    left_join(durations) %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(
      throughput = sum(success) / as.numeric(duration),
      .groups = "drop"
    ) %>%
    group_by(folder, metric_group) %>%
    summarise(
      p99_throughput = quantile(throughput, 0.99, na.rm = TRUE),
      .groups = "drop"
    ) %>%
    extract_context() %>%
    group_by(env_live) %>%
    summarise(
      avg_p99_throughput = mean(p99_throughput, na.rm = TRUE),
      .groups = "drop"
    ) %>%
    arrange(env_live)

  Log(p99_df)

  p99_df %>%
    mutate(file_name = paste0("p99_throughput_", env_live, ".txt")) %>%
    rowwise() %>%
    group_walk(
      ~ write(
        round(.x$avg_p99_throughput, 1),
        file = paste0("figures/", .x$file_name)
      )
    )

  df <- fallbacks_processed %>%
    filter(env_live %in% c(SCE_ONE, SCE_TWO)) %>%
    mutate(
      env_live = factor(env_live, levels = c(SCE_ONE, SCE_TWO))
    ) %>%
    ungroup() %>%
    complete(status, env_live, env, fill = list(n = NA)) %>%
    group_by(status, env_live, env) %>%
    summarise(avg = mean(n)) %>%
    filter(status %in% c("Total", "0 fallback")) %>%
    filter(env == LOAD_ONE)

  Log(df)

  inc_success_with_fallbacks <- df[2, ]$avg - df[4, ]$avg
  write(
    paste0(round(inc_success_with_fallbacks * 100, 1), " percentage points"),
    file = "figures/pressure_increase_with_fallbacks.txt"
  )
}
