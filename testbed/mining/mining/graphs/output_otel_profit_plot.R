output_otel_profit_plot <- function(spans) {
  # Compute slope for each service.namespace group to determine trend
  # Calculate slope for each group

  # Log(colnames(spans))

  # Log(spans %>% select(timestamp, budget) %>% filter(is.na(new_budget)))
  trend_data <- spans %>%
    filter(!is.na(budget)) %>%
    group_by(folder, service.namespace) %>%
    mutate(timestamp_numeric = as.numeric(timestamp)) %>%
    do(model = lm(budget ~ timestamp_numeric, data = .)) %>%
    mutate(
      slope = coef(model)["timestamp_numeric"],
      trend = ifelse(slope > 0, "increasing", "decreasing")
    ) %>%
    select(folder, service.namespace, trend, slope)

  Log(trend_data)

  # Merge trend back to original data
  spans <- spans %>%
    left_join(trend_data, by = c("folder", "service.namespace"))

  ggplot(
    data = spans,
    aes(
      x = service.namespace,
      y = budget,
      color = trend,
    )
  ) +
    geom_beeswarm() +
    labs(
      title = paste(
        "Budget evolution of each application\n",
        unique(spans$folder)
      ),
      x = "Service Namespace",
      y = "Budget",
    ) +
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
    )
}
