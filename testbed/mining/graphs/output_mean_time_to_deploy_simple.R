output_mean_time_to_deploy_simple <- function(raw.deployment_times) {
  df <- raw.deployment_times

  p <- ggplot(
    data = df,
    aes(x = docker_fn_name, y = value, color = folder, alpha = 1)
  ) +
    #  facet_grid(~var_facet) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Mean time to deploy (ms)",
      x = "Placement method",
    ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    scale_y_continuous(trans = "log10") +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
    ) +
    # theme(legend.position = c(.8, .5)) +
    guides(colour = guide_legend(ncol = 1)) +
    scale_color_viridis(discrete = T) +
    scale_fill_viridis(discrete = T) +
    geom_quasirandom(method = "tukey", alpha = .2)
  return(p)
}
