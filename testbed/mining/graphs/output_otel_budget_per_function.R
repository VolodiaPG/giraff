output_otel_budget_per_function_plot <- function(spans) {
  # Log(colnames(spans))
  #
  # x <- spans %>%
  #   select(where(~ !all(is.na(.))))
  #
  # Log(colnames(x))

  spans_budget <- spans %>%
    group_by(folder, service.namespace) %>%
    select(folder, service.namespace, budget, timestamp) %>%
    filter(!is.na(budget)) %>%
    arrange(timestamp) %>%
    summarise(budget = last(budget) - first(budget))

  Log(spans_budget)

  spans <- spans %>%
    select(
      folder,
      service.namespace,
      trace_id
    ) %>%
    ungroup() %>%
    group_by(folder, service.namespace) %>%
    distinct() %>%
    summarise(requests = n()) %>%
    left_join(spans_budget) %>%
    mutate(count = budget / requests)

  # Log(spans)

  #
  # Log(spans %>% ungroup() %>% select(service.instance.id))
  # summarigje(count = n())

  # Log(spans %>% select(gunctionImage, functionLiveName))

  ggplot(
    data = spans,
    aes(
      x = service.namespace,
      y = count,
      fill = budget >= 0
      # color = span.name,
    ),
  ) +
    geom_col(
      # position = position_dodge2()
    ) +
    # geom_beeswarm() +
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
