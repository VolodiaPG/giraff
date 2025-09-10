big_output_otel_fallbacks_plot <- function(spans, degrades, errors) {
  Log(degrades %>% filter(is.na(fallbacks)))
  df <- spans %>%
    # group_by(folder, metric_group, service.namespace,n e) %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    mutate(
      otel_error = ifelse(
        is.na(otel.status_code),
        FALSE,
        otel.status_code == "Error"
      )
    ) %>%
    select(folder, metric_group, trace_id, otel_error) %>%
    left_join(degrades) %>%
    left_join(
      errors %>% select(folder, metric_group, trace_id) %>% mutate(error = TRUE)
    ) %>%
    mutate(
      status = case_when(
        fallbacks > 0 ~ paste("Degraded,", fallbacks, "fallbacks"),
        fallbacks == 0 ~ "Nominal",
        error ~ "Failure"
      )
    ) %>%
    extract_context()
  # select(
  #   folder,
  #   metric_group,
  #   service.namespace,
  #   trace_id,
  #   error,
  #   fallbacks,
  #   status
  # )

  Log(colnames(df))

  ggplot(
    data = df,
    aes(
      x = env_live,
      fill = status
    ),
  ) +
    facet_grid(rows = vars(env)) +
    geom_bar() +
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
