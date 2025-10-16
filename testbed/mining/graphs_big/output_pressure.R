big_output_pressure_plot <- function(
  nb_requests,
  nb_nodes
) {
  df <- nb_requests %>%
    group_by(folder, metric_group) %>%
    summarise(requests = sum(success), .groups = "drop") %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    env_live_extract() %>%
    categorize_nb_nodes() %>%
    extract_env_name()

  df_mean <- df %>%
    group_by(env_live, nb_nodes, env) %>%
    summarise(requests = mean(requests))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = requests,
    )
  ) +
    facet_grid(cols = vars(env)) +
    geom_col(
      data = df_mean,
      aes(fill = env_live),
      alpha = 0.8,
      position = position_dodge(width = 0.9)
    ) +
    geom_point(
      aes(color = env_live),
      position = position_dodge(width = 0.9)
    ) +
    guides(group = "none") +
    labs(
      x = "Number of nodes",
      y = "Number of successful requests",
      fill = "Application Configuration",
      color = "Application Configuration"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
