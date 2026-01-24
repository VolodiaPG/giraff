big_output_otel_budget_plot <- function(
  profit,
  nb_requests,
  nb_nodes,
  nb_functions
) {
  df <- profit %>%
    left_join(nb_requests) %>%
    extract_context() %>%
    extract_env_name() %>%
    env_live_extract() %>%
    mutate(profit_per_request = profit / total)

  ggplot(
    data = df,
    aes(
      # x = requests,
      y = profit_per_request,
      color = env_live,
      group = folder,
    )
  ) +
    # geom_point() +
    stat_ecdf() +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
  # scale_x_continuous(labels = scales::percent) +
  # scale_y_log10()
}
