load_nb_requests <- function(spans) {
  spans %>%
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
    group_by(folder, metric_group, service.namespace) %>%
    summarise(
      requests = n(),
      otel_errors = sum(otel_error),
      success = requests - otel_errors
    )
}
