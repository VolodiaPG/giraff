output_jains_index_plot <- function(earnings.jains.plot.data.raw) {
  df <- earnings.jains.plot.data.raw %>%
    mutate(toto = "toto") %>%
    ungroup()
  p <- ggplot(data = df, aes(alpha = 1)) +
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
    theme(legend.position = "none") +
    # theme(legend.position = "top", legend.box = "vertical") +
    scale_color_viridis(discrete = T) +
    scale_fill_viridis(discrete = T) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(0, -10, -10, -10),
    )

  plots.jains.w <- GRAPH_ONE_COLUMN_WIDTH
  plots.jains.h <- GRAPH_ONE_COLUMN_HEIGHT
  plots.jains.caption <- "Jain's index at different ratio of low level latencies"
  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1f$}", Letters, mean))
  }
  plots.jains <- anova_boxplot(
    p,
    df,
    "Placement method",
    "score",
    "toto",
    mean_cb
  )
  plots.jains + labs(title = plots.jains.caption)
  return(plots.jains)
}
