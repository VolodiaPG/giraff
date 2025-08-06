output_mean_time_to_deploy <- function(raw.deployment_times) {
  df <- raw.deployment_times %>%
    group_by(folder, metric_group) %>%
    summarise(value = mean(value)) %>%
    correct_names() %>%
    mutate(group = "sdlkfjh") %>%
    ungroup()

  p <- ggplot(data = df, aes(alpha = 1)) +
    #  facet_grid(~var_facet) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Mean time to deploy (s)",
      x = "Placement method",
    ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2, color = alpha("white", .7)
      ),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
    # theme(legend.position = c(.8, .5)) +
    guides(colour = guide_legend(ncol = 1)) +
    scale_color_viridis(discrete = T) +
    scale_fill_viridis(discrete = T)
  plots.deploymenttimes.w <- GRAPH_ONE_COLUMN_WIDTH
  plots.deploymenttimes.h <- GRAPH_ONE_COLUMN_HEIGHT
  plots.deploymenttimes.caption <- "Time to find a fog node for a function"
  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1fs$}", Letters, mean))
  }
  plots.deploymenttimes <- anova_boxplot(p, df, "Placement method", "value", "group", mean_cb, c(4, 6, 19))
  plots.deploymenttimes + labs(title = plots.deployment_times.caption)
  return(plots.deploymenttimes)
}