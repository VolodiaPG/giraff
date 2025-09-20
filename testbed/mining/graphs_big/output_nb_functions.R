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
    categorize_nb_nodes()

  df_mean <- df %>%
    group_by(env, nb_nodes) %>%
    summarise(
      nb_functions = mean(nb_functions),
      max_nb_functions = max(max_nb_functions),
      min_nb_functions = min(min_nb_functions)
    )

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      group = env
    )
  ) +
    # geom_ribbon(
    #   data = df_mean,
    #   aes(ymin = min_nb_functions, ymax = max_nb_functions, fill = env),
    #   alpha = 0.2
    # ) +
    geom_col(
      data = df_mean,
      aes(y = nb_functions, fill = env),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(
      aes(y = nb_functions, color = env),
      position = position_dodge(width = 0.9)
    ) +
    geom_errorbar(
      data = df_mean,
      aes(
        x = nb_nodes,
        ymin = min_nb_functions,
        ymax = max_nb_functions,
        color = env
      ),
      position = position_dodge(width = 0.9),
      width = 0.2
    ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      # axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    labs(
      title = "Number of functions per application, depending on the number of nodes in the continuum",
      x = "Number of nodes",
      y = "Number of functions"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
