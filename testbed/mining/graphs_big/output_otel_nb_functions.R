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
    mutate(nb_nodes = factor(nb_nodes)) %>%
    extract_env_name()

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
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0)
    ) +
    labs(
      x = "Number of nodes",
      y = "Total number of functions"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
