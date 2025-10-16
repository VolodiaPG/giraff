load_otel_duration <- function(spans) {
  spans %>%
    extract_context() %>%
    filter(span.name == "start_processing_requests") %>%
    group_by(folder, service.namespace) %>%
    summarise(duration = max(timestamp) - min(timestamp))
}
