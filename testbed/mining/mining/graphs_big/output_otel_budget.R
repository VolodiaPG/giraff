big_output_otel_budget_plot <- function(spans) {
  spans_budget <- spans %>%
    group_by(folder, metric_group, service.namespace) %>%
    select(folder, metric_group, service.namespace, budget, timestamp) %>%
    filter(!is.na(budget)) %>%
    arrange(timestamp) %>%
    summarise(budget = last(budget) - first(budget))

  # Log(spans_budget)

  spans <- spans %>%
    select(
      folder,
      service.namespace,
      trace_id,
      metric_group
    ) %>%
    ungroup() %>%
    group_by(folder, metric_group, service.namespace) %>%
    distinct() %>%
    summarise(requests = n()) %>%
    inner_join(spans_budget) %>%
    mutate(count = budget / requests) %>%
    extract_context() %>%
    group_by(folder, env_live) %>%
    summarise(count = mean(count))

  # Log(spans)

  #
  # Log(spans %>% ungroup() %>% select(service.instance.id))
  # summarigje(count = n())

  # Log(spans %>% select(gunctionImage, functionLiveName))

  ggplot(
    data = spans,
    aes(
      x = env_live,
      y = count,
      fill = env_live
      # color = span.name,
    ),
  ) +
    geom_col(
      # position = position_dodge2()
    ) +
    # geom_beeswarm() +
    # labs(
    #   title = paste(
    #     "Budget evolution of each application\n",
    #     # unique(spans$folder)
    #   ),
    #   x = "Service Namespace",
    #   y = "Budget",
    # ) +
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
}
