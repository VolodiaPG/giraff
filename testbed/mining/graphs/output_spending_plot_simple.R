output_spending_plot_simple <- function(bids_won, node_levels) {
  df <- bids_won %>%
    extract_function_name_info() %>%
    left_join(node_levels %>% rename(winner = name))
  #   group_by(folder, metric_group, metric_group_group, level_value) %>%
  #   summarise(cost = mean(cost))
  p <- ggplot(data = df, aes(x = winner, y = cost, alpha = 1)) +
    theme(legend.position = "none") +
    # facet_grid(rows = vars(level_value)) +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Function cost",
      x = "Placement method",
    ) +
    theme(legend.background = element_rect(
      fill = alpha("white", .7),
      size = 0.2, color = alpha("white", .7)
    )) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    guides(colour = guide_legend(nrow = 1)) +
    geom_quasirandom(aes(color = factor(level_value)), method = "tukey", alpha = .2)
  # geom_line(aes(group = metric_group_group), alpha = .2) +
  # geom_point(aes(color = metric_group, fill = metric_group, )) +
  # stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)
  return(p)
}