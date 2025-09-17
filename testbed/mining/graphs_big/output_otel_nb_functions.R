big_output_otel_nb_functions_plot <- function(
  functions,
  nb_nodes
) {
  # Log(functions)
  # Log(colnames(functions))

  df <- functions %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group) %>%
    summarise(total = sum(n)) %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    mutate(nb_nodes = factor(nb_nodes))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = total,
      color = env,
      group = folder
    )
  ) +
    geom_beeswarm() +
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
      y = "Total number of functions"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
