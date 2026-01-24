load_fallbacks_processed <- function(
  spans,
  errors
) {
  spans <- spans %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    filter(duration > 0) %>%
    mutate(
      otel_error = otel.status_code == "Error"
    ) %>%
    select(folder, metric_group, service.namespace, otel_error, trace_id) %>%
    distinct()

  # df <- df %>%
  # inner_join(
  df <- errors %>%
    select(
      folder,
      metric_group,
      trace_id,
      fallbacks,
      timeout,
      error,
      is_fallback
    ) %>%
    distinct() %>%
    full_join(
      spans,
      by = c("folder", "metric_group", "trace_id")
    )

  df <- df %>%
    mutate(
      status = case_when(
        is_fallback ~ paste0(
          fallbacks,
          ifelse(fallbacks > 1, " fallbacks", " fallback")
        ),
        timeout ~ "Failure",
        otel_error ~ "Failure",
        error ~ "Failure",
        # TRUE ~ "0 fallback"
        # TRUE ~ "Failure"
        TRUE ~ NA
      )
    ) %>%
    filter(!is.na(status))
  # group_by(metric_group, folder, trace_id) %>%
  # complete(
  #   status,
  #   fill = list(n = 0)
  # ) %>%
  # collect()

  df
}
