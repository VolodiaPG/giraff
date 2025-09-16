big_output_nb_functions_plot <- function(spans, node_levels) {
  # Log(colnames(spans))

  nb_nodes <- node_levels %>%
    group_by(folder) %>%
    summarise(nb_nodes = n())

  df <- spans %>%
    extract_context() %>%
    select(
      env,
      env_live,
      folder,
      metric_group,
      service.namespace,
      service.instance.id,
      run
    ) %>%
    group_by(run, folder, metric_group, service.namespace, env, env_live) %>%
    distinct() %>%
    summarise(nb_functions = n()) %>%
    left_join(nb_nodes, by = c("folder"))

  ggplot(
    data = df,
    aes(
      x = factor(nb_nodes),
      y = nb_functions,
      color = folder
    ),
  ) +
    # facet_grid(vars(un)) +
    geom_quasirandom(method = "tukey") +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.position = "none",
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    labs(
      title = "Number of functions per application, depending on the number of nodes in the continuum",
      x = "Number of nodes",
      y = "Number of functions"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
