load_otel_processed <- function(otel) {
  otel %>%
    ungroup() %>%
    select(
      folder,
      metric_group,
      timestamp,
      service.name,
      field,
      value_raw,
      span_id,
      trace_id
    ) %>%
    group_by(folder) %>%
    pivot_wider(names_from = field, values_from = value_raw) %>%
    mutate(
      duration = as.difftime(as.numeric(duration_nano) / 1e9, unit = "secs")
    ) %>%
    mutate(attributes = lapply(attributes, fromJSON)) %>%
    unnest_wider(attributes) %>%
    mutate(end_timestamp = timestamp + duration)
}
