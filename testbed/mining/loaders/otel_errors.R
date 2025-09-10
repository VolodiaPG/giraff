load_otel_errors <- function(otel_logs) {
  otel_logs <- otel_logs %>%
    ungroup() %>%
    select(
      folder,
      metric_group,
      timestamp,
      attributes,
      span_id,
      trace_id,
      event.name
    ) %>%
    filter(event.name == "log") %>%
    filter(str_detect(attributes, "Request processing error"))

  logs <- stringr::str_match(
    otel_logs$attributes,
    "Request processed, with ([0-9]+) fallbacks"
  )

  otel_logs <- otel_logs %>%
    mutate(fallbacks = as.numeric(logs[, 2])) %>%
    select(-attributes)

  # filter(!is.na(fallbacks))

  otel_logs
}
