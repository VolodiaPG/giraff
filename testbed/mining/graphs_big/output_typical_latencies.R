big_ouput_typical_latencies_plot <- function(func_with_latencies) {
  df <- func_with_latencies %>%
    extract_flame_function_name() %>%
    extract_context() %>%
    ungroup() %>%
    group_by(
      folder,
      metric_group,
      service.namespace,
      env,
      env_live,
      span.name
    ) %>%
    summarise(latency = mean(latency))

  ggplot(
    data = df,
    aes(
      x = span.name,
      y = latency,
      color = folder
    ),
  ) +
    facet_grid(rows = vars(env), cols = vars(env_live)) +
    # geom_beeswarm() +
    geom_quasirandom(method = "tukey") +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.position = "none",
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    labs(
      title = paste("Typical Latencies of Functions"),
      x = "Function",
      y = "Latency (s)"
    )
}
