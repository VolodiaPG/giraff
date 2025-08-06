output_raw_cpu_usage <- function(raw_cpu) {
  df <- raw_cpu

  p <- ggplot(data = df, aes(x = timestamp, y = used, color = folder, alpha = 1)) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Mean time to deploy (ms)",
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
    scale_fill_viridis(discrete = T) +
    geom_quasirandom(method = "tukey", alpha = .2)
  return(p)
}