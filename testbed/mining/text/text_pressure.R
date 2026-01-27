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
    group_by(env_live, env) %>%
    summarise(requests = mean(requests)) %>%
    arrange(env_live, env)

  df_main <- df %>%
    filter(env_live %in% c(SCE_ONE, SCE_TWO))

  inc_success <- df_main[3, ]$requests - df_main[1, ]$requests
  write(
    paste0(round(inc_success * 100, 1), "\\text{ percentage points}"),
    file = "figures/pressure_increase.txt"
  )

  inc_success <- df_main[3, ]$requests - df_main[4, ]$requests
  write(
    paste0(round(inc_success * 100, 1), "\\text{ percentage points}"),
    file = "figures/pressure_increase_btw_loads.txt"
  )

  df_ccc <- df %>%
    filter(env_live %in% c(SCE_TWO, SCE_THREE))

  inc_success <- df[3, ]$requests - df[1, ]$requests
  write(
    paste0(round(inc_success * 100, 1), "\\text{ percentage points}"),
    file = "figures/pressure_increase_ccc.txt"
  )

  write(
    paste0(round(df_ccc[3, ]$requests * 100, 1), "\\%"),
    file = "figures/success_flavor_three.txt"
  )
  Log(colnames(fallbacks_processed))

  total <- nb_requests %>%
    group_by(folder) %>%
    summarise(nb_functions = n())

  df <- fallbacks_processed %>%
    filter(env_live == SCE_TWO) %>%
    filter(env == LOAD_ONE) %>%
    filter(status %in% c("1 fallback", "2 fallbacks")) %>%
    left_join(nb_requests, by = c("folder", "service.namespace")) %>%
    mutate(n = n / total) %>%
    group_by(folder, status, env, env_live, run) %>%
    summarise(n = sum(n)) %>%
    left_join(total, by = c("folder")) %>%
    mutate(n = n / nb_functions) %>%
    group_by(status, env, env_live) %>%
    summarise(n = mean(n))

  write(
    paste0(round(df[1, ]$n * 100, 1), "\\%"),
    file = "figures/success_one_fallbacks_flavor_two.txt"
  )

  write(
    paste0(round(df[2, ]$n * 100, 1), "\\%"),
    file = "figures/success_two_fallbacks_flavor_two.txt"
  )
}
