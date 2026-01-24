text_pressure_output <- function(nb_requests, durations, fallbacks_processed) {
  df <- nb_requests %>%
    group_by(folder, metric_group) %>%
    summarise(
      success = sum(success),
      total = sum(total),
      .groups = "drop"
    ) %>%
    mutate(requests = success / total) %>%
    extract_context() %>%
    env_live_extract() %>%
    extract_env_name() %>%
    filter(env_live %in% c(SCE_ONE, SCE_TWO)) %>%
    group_by(env_live, env) %>%
    summarise(requests = mean(requests)) %>%
    arrange(env_live, env)

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
