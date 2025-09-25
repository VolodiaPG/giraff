big_output_otel_nb_requests_plot <- function(
  nb_requests,
  nb_nodes
) {
  df <- nb_requests %>%
    extract_context() %>%
    group_by(folder, env) %>%
    summarise(requests = sum(requests)) %>%
    left_join(nb_nodes, by = c("folder")) %>%
    extract_env_name() %>%
    categorize_nb_nodes()

  df_mean <- df %>%
    group_by(env, nb_nodes) %>%
    summarise(requests = mean(requests))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      group = env
    )
  ) +
    geom_col(
      data = df_mean,
      aes(y = requests, fill = env),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(
      aes(y = requests, color = env),
      position = position_dodge(width = 0.9)
    ) +
    theme(
      axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    ) +
    labs(
      x = "Number of nodes",
      y = "Number of requests",
      fill = "Load",
      color = "Load"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
