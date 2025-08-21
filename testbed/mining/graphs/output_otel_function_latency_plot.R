output_otel_function_latency_plot <- function(latencies) {
  ggplot(
    data = latencies,
    aes(
      color = span.name,
      x = as.numeric(latency),
      group = span.name
    )
  ) +
    stat_ecdf() +
    scale_alpha_continuous(guide = "none") +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        # size = 0.2,
        color = alpha("white", .7)
      )
    ) +
    labs(
      x = "Latency (s)",
      y = "Percentage of functions",
      title = paste("Function latency distribution\n", unique(latencies$folder))
    ) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    guides(colour = guide_legend(nrow = 2))
}
