big_output_otel_fallbacks_plot <- function(
  spans,
  degrades,
  errors,
  node_levels
) {
  Log(errors %>% select(fallbacks, error))
  nb_nodes <- node_levels %>%
    group_by(folder) %>%
    summarise(nb_nodes = n())

  requests <- spans %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    mutate(
      otel_error = ifelse(
        is.na(otel.status_code),
        FALSE,
        otel.status_code == "Error"
      )
    ) %>%
    select(folder, service.namespace, metric_group, trace_id, otel_error) %>%
    distinct()

  Log(
    requests %>%
      select(folder, service.namespace, trace_id) %>%
      group_by(folder, service.namespace) %>%
      summarise(total = n())
  )

  df <- requests %>%
    # left_join(degrades) %>%
    left_join(
      errors
    ) %>%
    mutate(
      status = case_when(
        otel_error ~ "Failure",
        fallbacks > 0 ~ paste("Succes, Degraded with", fallbacks, "fallbacks"),
        fallbacks == 0 ~ "Success, Nominal"
      )
    ) %>%
    group_by(
      folder,
      metric_group,
      service.namespace,
      status
    ) %>%
    summarise(n = n(), .groups = "drop") %>%
    left_join(
      requests %>%
        select(folder, service.namespace, trace_id) %>%
        group_by(folder, service.namespace) %>%
        summarise(total = n())
    ) %>%
    mutate(n = n / total) %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    mutate(nb_nodes = factor(nb_nodes))

  Log(df %>% filter(status == "Nominal" & n < 0.1))

  ggplot(
    data = df %>% ungroup() %>% filter(!is.na(status)),
    aes(
      x = nb_nodes,
      y = n,
      fill = env,
      group = folder
    )
  ) +
    facet_grid(cols = vars(status)) +
    geom_boxplot(position = position_dodge()) +
    theme(
      legend.position = "none",
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0)
    ) +
    scale_y_continuous(labels = scales::percent) +
    labs(
      x = "Number of nodes",
      y = "Proportion of requests"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
