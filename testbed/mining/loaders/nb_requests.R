load_nb_requests <- function(spans_processed) {
  df <- spans_processed %>%
    select(
      folder,
      service.namespace,
      metric_group,
      trace_id,
      otel_error,
      status
    ) %>%
    group_by(folder, metric_group, service.namespace) %>%
    filter(!is.na(status)) %>%
    summarise(
      requests = n(),
      otel_errors = sum(status == "Failure"),
      success = sum(status != "Failure"),
      .groups = "drop"
    )
  df
}
