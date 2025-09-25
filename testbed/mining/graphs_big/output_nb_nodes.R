big_output_nb_nodes_plot <- function(node_levels) {
  df <- node_levels %>%
    mutate(alpha = FALSE) %>%
    extract_context() %>%
    group_by(folder, run, metric_group, level_value, alpha) %>%
    summarise(n = n()) %>%
    group_by(run, level_value, alpha) %>%
    summarise(n = mean(n)) %>%
    mutate(level_value = as.character(level_value)) %>%
    filter(!is.na(run))

  total <- df %>%
    group_by(run) %>%
    summarise(nb_nodes = sum(n)) %>%
    categorize_nb_nodes()

  df <- df %>%
    left_join(total)

  ggplot(
    data = df,
    aes(
      y = n,
      x = run,
      fill = level_value,
      # alpha = alpha
      group = run
    ),
  ) +
    facet_grid(cols = vars(nb_nodes), scales = "free_x") +
    geom_col(alpha = 0.8) +
    theme(
      axis.text.x = element_blank()
    ) +
    labs(
      # title = paste("Number of nodes for each layer of the continuum"),
      x = "Runs",
      y = "Nodes (VMs)",
      color = "Node level",
      fill = "Node level"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    scale_alpha_discrete(range = c(0.5, 0.9))
}
