output_jains <- function(earnings.jains.plot.data.raw) {
  plots.jains.w <- GRAPH_ONE_COLUMN_WIDTH
  plots.jains.h <- GRAPH_ONE_COLUMN_HEIGHT
  plots.jains.caption <- "Jain's index at different ratio of low level latencies"

  my_comparisons <- combn(
    unique(earnings.jains.plot.data.raw$`Placement method`),
    2
  )
  my_comparisons <- apply(my_comparisons, 2, list)
  my_comparisons <- lapply(my_comparisons, unlist)
  plots.jains <- earnings.jains.plot.data.raw %>%
    ggplot(aes(
      alpha = 1,
      x = `Placement method`,
      y = score,
      fill = `Placement method`,
      color = `Placement method`
    )) +
    geom_hline(
      yintercept = max(earnings.jains.plot.data.raw$worst_case),
      color = "black"
    ) +
    annotate(
      "text",
      x = "\footnotesize{Edge\\dash{}furthest}",
      y = max(earnings.jains.plot.data.raw$worst_case) + .05,
      label = sprintf(
        "$max(1/n)=%s$",
        max(earnings.jains.plot.data.raw$worst_case)
      ),
      color = "black"
    ) +
    geom_beeswarm() +
    # stat_compare_means(comparisons = my_comparisons, label = "p.signif") +
    # stat_anova_test() +
    labs(
      x = "Placement method",
      y = "Jain's index"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(
      color = guide_legend(nrow = 1),
      shape = guide_legend(nrow = 1),
      size = guide_legend(nrow = 1)
    ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = "white"
      ),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    theme(legend.position = "top", legend.box = "vertical") +
    scale_color_viridis(discrete = T) +
    scale_fill_viridis(discrete = T) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(0, -10, -10, -10),
    )

  plots.jains + labs(title = plots.jains.caption)
  return(plots.jains)
}
