output_faults_per_function_plot_simple <- function(respected_sla) {
  df <- respected_sla %>%
    mutate(ran_for = as.numeric(ran_for)) %>%
    mutate(some_not_acceptable = acceptable + all_errors != total)

  p <- ggplot(data = df, aes(x = interaction(prev_function, docker_fn_name), y = all_errors, color = some_not_acceptable, alpha = 1)) +
    #  facet_grid(~var_facet) +
    theme(legend.background = element_rect(
      fill = alpha("white", .7),
      size = 0.2, color = alpha("white", .7)
    )) +
    theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
    theme(axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)) +
    # scale_y_continuous(trans = "log10") +
    # theme(legend.position = "none") +
    scale_color_viridis(discrete = T) +
    scale_fill_viridis(discrete = T) +
    # scale_y_continuous(labels = scales::percent) +
    labs(
      x = "Placement method",
      y = "mean ran_for (s)"
    ) +
    geom_quasirandom(method = "tukey", alpha = .2)

  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
  }
  return(p)
}