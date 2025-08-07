output_request_distribution <- function(respected_sla) {
  df <- respected_sla
  p <- ggplot(
    data = df,
    aes(x = total, y = acceptable, color = docker_fn_name, alpha = 1)
  ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    labs(
      x = "function",
      y = "number of requests"
    ) +
    geom_point() +
    geom_line()

  return(p)
}
