load_nb_requests <- function(spans, errors) {
  df <- spans %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    mutate(
      otel_error = if ("otel.status_code" %in% names(df)) {
        ifelse(
          is.na(otel.status_code),
          FALSE,
          otel.status_code == "Error"
        )
      } else {
        FALSE # Default value if column does not exist
      }
    ) %>%
    select(folder, service.namespace, metric_group, trace_id, otel_error) %>%
    distinct() %>%
    left_join(
      errors
    ) %>%
    mutate(
      status = case_when(
        # timeout ~ "Timeout",
        otel_error ~ "Failure",
        error ~ "Failure",
        fallbacks == 1 ~ "Succes, Degraded with 1 fallback",
        fallbacks == 2 ~ "Succes, Degraded with 2 fallbacks",
        TRUE ~ "Success, Nominal",
      ),
    ) %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(
      requests = n(),
      otel_errors = sum(status == "Failure"),
      success = sum(status != "Failure"),
      .groups = "drop"
    )
}
