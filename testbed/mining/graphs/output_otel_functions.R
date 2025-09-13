output_otel_functions_plot <- function(processed, spans, log_errors) {
  # Log(colnames(log_errors))

  Log(log_errors %>% filter(error))

  errors <- processed %>%
    filter(startsWith(span.name, "start_processing_requests")) %>%
    select(folder, service.namespace, trace_id, otel.status_code) %>%
    mutate(
      otel_error = ifelse(
        is.na(otel.status_code),
        FALSE,
        otel.status_code == "Error"
      )
    ) %>%
    full_join(log_errors, by = c("folder", "trace_id")) %>%
    group_by(folder, service.namespace) %>%
    summarise(
      otel_errors = sum(otel_error),
      log_errors = sum(error),
      timeouts = sum(timeout & error)
    )

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
    group_by(folder, service.namespace, span.name) %>%
    distinct() %>%
    summarise(requests = n()) %>%
    left_join(errors)

  # group_by(folder, service.namespace, span.name) %>%
  # summarise(count = requests / n(), errors = errors / n())
  #
  # Log(spans %>% ungroup() %>% select(requests, errors))
  # summarigje(count = n())

  # Log(spans %>% select(gunctionImage, functionLiveName))

  # Log(errors %>% select(errors))

  ggplot(spans, aes(x = service.namespace)) +
    geom_col(
      aes(
        y = requests,
        fill = span.name,
      ),
    ) +
    geom_segment(
      aes(
        xend = service.namespace,
        y = 0,
        yend = otel_errors
      ),
      size = 2,
      color = "red"
    ) +
    # geom_segment(
    #   aes(
    #     xend = service.namespace,
    #     y = 0,
    #     yend = log_errors
    #   ),
    #   color = "black",
    #   size = 2,
    # ) +
    geom_segment(
      aes(
        xend = service.namespace,
        y = 0,
        yend = timeouts
      ),
      color = "orange"
    ) +
    labs(
      title = paste(
        unique(spans$folder)
      )
    ) +

    # geom_beeswarm() +
    # labs(
    #   title = paste(
    #     "Budget evolution of each application\n",
    #     unique(spans$folder)
    #   ),
    #   # x = "Service Namespace",
    #   # y = "Budget",
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
    )
}
