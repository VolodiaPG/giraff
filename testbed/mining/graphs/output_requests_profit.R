output_requests_profit_plot <- function(
  profit,
  nb_requests,
  nb_nodes,
  nb_functions
) {
  Log(colnames(nb_requests))
  df <- profit %>%
    left_join(
      nb_requests %>%
        mutate(requests = success / requests) %>%
        select(folder, requests)
    ) %>%
    left_join(nb_functions)

  # mutate(benefit = ifelse(profit >= 0, "Profit", "Loss")) %>%
  # left_join(nb_functions)
  # mutate(profit = profit / requests)
  # mutate(profit = profit / requests / nb_functions)
  # mutate(profit = profit)
  #
  # center <- df %>%
  #   group_by(folder) %>%
  #   summarise(
  #     mean = mean(profit),
  #     sd = sd(profit),
  #     mean_requests = mean(requests),
  #     sd_requests = sd(requests),
  #     mean_nb_functions = mean(nb_functions),
  #     sd_nb_functions = sd(nb_functions),
  #     nb = n()
  #   ) %>%
  #   mutate(
  #     se = sd_requests / sqrt(nb),
  #     lower.ci = mean_requests - qnorm(0.975) * se,
  #     upper.ci = mean_requests + qnorm(0.975) * se
  #   )

  df <- df %>%
    # left_join(center) %>%
    # rowwise() %>%
    # mutate(
    #   ci = case_when(
    #     profit < lower.ci ~ "below",
    #     profit > upper.ci ~ "above",
    #     TRUE ~ "within"
    #   ),
    #   ci = factor(ci, levels = c("below", "within", "above"))
    # ) %>%
    # mutate(profit = (profit - mean) / sd) %>%
    # mutate(requests = (requests - mean_requests) / sd_requests) %>%
    # mutate(
    #   nb_functions = (nb_functions - mean_nb_functions) / sd_nb_functions
    # ) %>%
    left_join(nb_nodes) %>%
    extract_context() %>%
    filter(env_live != 4) %>%
    env_live_extract() %>%
    extract_env_name() %>%
    group_by(run, nb_nodes, folder, env_live, env) %>%
    summarise(
      profit = mean(profit),
      requests = mean(requests),
      nb_functions = mean(nb_functions)
    )

  df_center <- df %>%
    group_by(run, env) %>%
    summarise(
      mean = mean(profit),
      sd = sd(profit),
      mean_requests = mean(requests),
      sd_requests = sd(requests),
      mean_nb_functions = mean(nb_functions),
      sd_nb_functions = sd(nb_functions),
    )

  df <- df %>%
    left_join(df_center) %>%
    mutate(
      profit = (profit - mean) / sd,
      requests = (requests - mean_requests) / sd_requests,
      nb_functions = (nb_functions - mean_nb_functions) / sd_nb_functions
    ) %>%
    group_by(run, nb_nodes, env_live, env) %>%
    summarise(
      profit = mean(profit),
      requests = mean(requests),
      nb_functions = mean(nb_functions)
    )

  df_runnb <- df %>%
    ungroup() %>%
    select(run, env) %>%
    distinct() %>%
    mutate(run_label = row_number())

  max_env_live <- df %>%
    select(env, run, env_live) %>%
    group_by(env, run) %>%
    summarise(nb_envlive = n(), .groups = "drop") %>%
    group_by(env) %>%
    summarise(max_env_live = max(nb_envlive), .groups = "drop")

  keep_runs <- df %>%
    ungroup() %>%
    select(run, env, env_live) %>%
    distinct() %>%
    group_by(run, env) %>%
    summarise(nb_run = n(), .groups = "drop") %>%
    left_join(max_env_live) %>%
    mutate(alpha = ifelse(nb_run == max_env_live, "Full", "Missing at least 1"))

  df <- df %>%
    left_join(df_runnb) %>%
    left_join(keep_runs) %>%
    ungroup() %>%
    filter(alpha == "Full")

  Log(df %>% select(run, nb_run, alpha))

  ggplot(
    data = df,
    aes(
      y = profit,
      x = requests,
      color = env_live,
      fill = env_live,
      # size = nb_nodes,
      # alpha = alph
      label = run_label,
    ),
  ) +
    # facet_grid(cols = vars(env)) +
    # stat_ellipse(type = "norm", geom = "polygon", alpha = .1) +
    geom_point(
      # aes(
      #   # size = nb_functions,
      #   # label = run_label,
      # ),
    ) +
    geom_text(hjust = 0, nudge_x = 0.02) +
    geom_hline(yintercept = 0, color = "black", linetype = "dotted") +
    geom_vline(xintercept = 0, color = "black", linetype = "dotted") +
    labs(
      x = "Centered-reduced ratio of successful requests",
      y = "Centered-reduced profit",
      size = "Number of nodes",
      color = "Application Configuration",
      fill = "Application Configuration",
      alpha = "Completness of the run"
    ) +
    scale_alpha_discrete(range = c(1, 0.35)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
