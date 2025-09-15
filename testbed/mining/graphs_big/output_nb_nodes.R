big_output_nb_nodes_plot <- function(node_levels) {
  df <- node_levels %>%
    extract_context() %>%
    group_by(run, metric_group, level_value) %>%
    summarise(n = n()) %>%
    group_by(run, level_value) %>%
    summarise(n = mean(n))

  ggplot(
    data = df,
    aes(
      x = level_value,
      y = n,
      fill = run
    ),
  ) +
    # facet_grid(rows = vars(env), cols = vars(env_live)) +
    # geom_beeswarm() +
    # geom_quasirandom(method = "tukey") +
    geom_col() +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.position = "none",
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10)
      # axis.text.x = element_text(angle k 0, vjust = 1, hjust = 1)
    ) +
    labs(
      title = paste("Number of nodes for each layer of the continuum"),
      x = "Node level",
      y = "Nodes (VMs)"
    )
}
