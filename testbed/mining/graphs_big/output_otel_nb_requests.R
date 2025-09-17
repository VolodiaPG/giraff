big_output_otel_nb_requests_plot <- function(
  nb_requests,
  nb_nodes
) {
  df <- nb_requests %>%
    extract_context() %>%
    group_by(folder, env) %>%
    summarise(requests = sum(requests)) %>%
    left_join(nb_nodes, by = c("folder")) %>%
    mutate(nb_nodes = factor(nb_nodes))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = requests,
      fill = env,
      group = folder
    )
  ) +
    geom_boxplot(position = position_dodge()) +
    theme(
      legend.position = "none",
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
      y = "Number of requests"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
