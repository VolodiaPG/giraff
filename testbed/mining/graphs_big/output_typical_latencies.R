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
    extract_context() %>%
    extract_function_name() %>%
    extract_env_name()

  all_max <- df %>%
    group_by(span.name) %>%
    summarise(max_latency = max(latency), min_latency = min(latency))

  df <- df %>%
    group_by(folder, metric_group, span.name) %>%
    summarise(latency = mean(latency))
  all_averages <- df %>%
    group_by(span.name) %>%
    summarise(latency = mean(latency))

  ggplot(
    data = df,
    aes(
      x = span.name,
    ),
  ) +
    geom_col(
      data = all_averages,
      aes(x = span.name, y = latency),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_beeswarm(
      aes(y = latency),
      position = position_dodge(width = 0.9),
      cex = 0.8,
      alpha = 0.5,
      size = 0.5,
    ) +
    geom_errorbar(
      data = all_max,
      aes(x = span.name, ymin = min_latency, ymax = max_latency),
      position = position_dodge(width = 0.9),
      width = 0.1,
    ) +
    theme(
      axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    ) +
    labs(
      x = "Function",
      y = "Latency (s)",
      color = "Load",
      fill = "Load",
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
