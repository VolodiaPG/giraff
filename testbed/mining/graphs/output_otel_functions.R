output_otel_functions_plot <- function(spans) {
  # Log(colnames(spans))

  x <- spans %>%
    select(where(~ !all(is.na(.))))

  Log(colnames(x))

  spans <- spans %>%
    select(
      folder,
      service.namespace,
      span.name,
      service.instance.id,
      trace_id
    ) %>%
    filter(startsWith(span.name, "FLAME")) %>%
    ungroup() %>%
    group_by(folder, service.namespace, span.name, service.instance.id) %>%
    distinct() %>%
    summarise(requests = n()) %>%
    group_by(folder, service.namespace, span.name) %>%
    summarise(count = requests / n())

  #
  # Log(spans %>% ungroup() %>% select(service.instance.id))
  # summarigje(count = n())

  # Log(spans %>% select(gunctionImage, functionLiveName))

  ggplot(
    data = spans,
    aes(
      x = service.namespace,
      y = count,
      # color = span.name,
      fill = span.name
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
