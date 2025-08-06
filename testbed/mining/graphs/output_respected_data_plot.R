output_respected_data_plot <- function(respected_sla) {
  df <- respected_sla %>%
    mutate(satisfied_ratio = acceptable_chained / total) %>%
    # group_by(folder, docker_fn_name, metric_group) %>%
    # summarise(satisfied_ratio = mean(satisfied_ratio) %>%
    ungroup()

  p <- ggplot(data = df, aes(alpha = 1, x = satisfied_ratio, color = docker_fn_name)) +
    facet_grid(rows = vars(metric_group)) +
    theme(legend.position = "none") +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    scale_y_continuous(labels = scales::percent) +
    # geom_quasirandom(method='tukey',alpha=.2) +
    # geom_boxplot() +
    stat_ecdf() +
    labs(
      x = "Satisfaction rate",
      y = "ecdf"
    )
  return(p)
}