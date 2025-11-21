big_output_nb_functions_plot <- function(nb_functions, nb_nodes) {
  df <- nb_functions %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(
      nb_functions = mean(nb_functions),
      max_nb_functions = max(nb_functions),
      min_nb_functions = min(nb_functions)
    ) %>%
    group_by(folder, metric_group) %>%
    summarise(
      nb_functions = mean(nb_functions),
      max_nb_functions = max(max_nb_functions),
      min_nb_functions = min(min_nb_functions)
    ) %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    # mutate(nb_nodes = factor(nb_nodes)) %>%
    extract_env_name() %>%
    # categorize_nb_nodes() %>%
    env_live_extract()

  df_mean <- df %>%
    group_by(env, nb_nodes, env_live) %>%
    summarise(
      nb_functions = mean(nb_functions),
      max_nb_functions = max(max_nb_functions),
      min_nb_functions = min(min_nb_functions)
    )

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = nb_functions,
      # group = env
      color = env_live,
      fill = env_live
    )
  ) +
    geom_beeswarm(alpha = 0.5, color = "black") +
    # geom_segment(
    #   data = df_mean,
    #   aes(
    #     x = nb_nodes - 1,
    #     xend = nb_nodes + 1,
    #     y = max_nb_functions,
    #     yend = max_nb_functions,
    #   ),
    #   size = 0.2
    # ) +
    # geom_segment(
    #   data = df_mean,
    #   aes(
    #     x = nb_nodes - 2,
    #     xend = nb_nodes + 2,
    #     y = min_nb_functions,
    #     yend = min_nb_functions,
    #   ),
    #   size = 0.2
    # ) +
    geom_smooth(
      method = "lm",
      se = TRUE,
      fullrange = TRUE,
      level = 0.95,
      alpha = 0.5
    ) +
    theme(
      axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    ) +
    labs(
      # title = "Number of functions per application, depending on the number of nodes in the continuum",
      x = "Number of nodes",
      y = "Number of functions",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
