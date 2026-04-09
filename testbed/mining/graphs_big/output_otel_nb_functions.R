big_output_otel_nb_functions_plot <- function(
  functions,
  nb_nodes
) {
  df <- functions %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group) %>%
    summarise(total = sum(n)) %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    extract_env_name() %>%
    env_live_extract()
  # categorize_nb_nodes()
  #
  # df_mean <- df %>%
  #   group_by(env, nb_nodes) %>%
  #   summarise(total = mean(total))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = total,
      color = env_live,
      group = env_live,
    )
  ) +
    facet_grid(cols = vars(env)) +
    geom_point(alpha = 0.8) +
    geom_smooth(
      aes(fill = env_live),
      method = "lm",
      se = TRUE,
      fullrange = TRUE,
      level = 0.95,
      alpha = 0.1,
      size = 0,
      show.legend = FALSE
    ) +
    geom_line(stat = "smooth", method = "lm", alpha = 0.3, size = 1) +
    guides(
      group = "none",
    ) +
    labs(
      x = "Number of nodes",
      y = "Total number of functions",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    scale_color_viridis(
      discrete = TRUE,
      guide = guide_legend(override.aes = list(size = 2, alpha = 1))
    ) +
    scale_fill_viridis(
      discrete = TRUE,
      guide = guide_legend(override.aes = list(size = 2, alpha = 1))
    )
}
