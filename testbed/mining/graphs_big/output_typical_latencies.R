big_output_typical_latencies_plot <- function(func_with_latencies) {
  df <- func_with_latencies %>%
    extract_flame_function_name() %>%
    ungroup() %>%
    group_by(
      folder,
      metric_group,
      service.namespace,
      span.name
    ) %>%
    summarise(latency = mean(latency)) %>%
    group_by(folder, metric_group, span.name) %>%
    summarise(latency = mean(latency)) %>%
    extract_context() %>%
    extract_function_name() %>%
    extract_env_name()

  all_averages <- df %>%
    group_by(span.name, env) %>%
    summarise(latency = mean(latency))

  ggplot(
    data = df,
    aes(
      x = span.name,
      y = latency,
      group = env
    ),
  ) +
    geom_col(
      data = all_averages,
      aes(x = span.name, y = latency, fill = env),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(position = position_dodge(width = 0.9), aes = aes(color = env)) +
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
    labs(
      title = paste("Typical Latencies of Functions"),
      x = "Function",
      y = "Latency (s)",
      color = "Load",
      fill = "Load",
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
