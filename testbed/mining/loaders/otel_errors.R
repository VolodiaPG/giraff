load_otel_errors <- function(otel_logs) {
  otel_logs <- otel_logs %>%
    ungroup() %>%
    select(
      folder,
      metric_group,
      attributes,
      trace_id,
      event.name
    ) %>%
    filter(event.name == "log") %>%
    select(-event.name)

  logs <- stringr::str_match(
    otel_logs$attributes,
    "Request processed, with ([0-9]+) fallbacks"
  )

  otel_logs <- otel_logs %>%
    mutate(error = str_detect(attributes, "Request processing error")) %>%
    mutate(timeout = str_detect(attributes, ":timeout")) %>%
    mutate(fallbacks = as.numeric(logs[, 2])) %>%
    select(-attributes) %>%
    group_by(folder, metric_group, trace_id) %>%
    summarise(
      error = any(error),
      fallbacks = sum(fallbacks),
      timeout = any(timeout)
    )

  # filter(!is.na(fallbacks))

  otel_logs
}
