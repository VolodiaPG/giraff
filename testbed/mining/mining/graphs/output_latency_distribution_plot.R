output_otel_duration_latency_plot <- function(provisioned_sla) {
  df <- provisioned_sla %>%
    extract_function_name_info()
  p <- ggplot(
    data = df,
    aes(x = docker_fn_name, y = latency, color = docker_fn_name, alpha = 1)
  ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    labs(
      x = "Placement method",
      y = "function required latency (s)"
    ) +
    geom_beeswarm()

  return(p)
}
