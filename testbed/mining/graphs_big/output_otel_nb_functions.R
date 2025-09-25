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
    categorize_nb_nodes()

  df_mean <- df %>%
    group_by(env, nb_nodes) %>%
    summarise(total = mean(total))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      group = env
    )
  ) +
    geom_col(
      data = df_mean,
      aes(y = total, fill = env),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(
      aes(y = total, color = env),
      position = position_dodge(width = 0.9)
    ) +
    theme(
      axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    ) +
    labs(
      x = "Number of nodes",
      y = "Total number of functions",
      fill = "Load",
      color = "Load"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
