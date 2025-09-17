load_nb_requests <- function(spans) {
  spans %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    select(folder, service.namespace, metric_group, trace_id) %>%
    distinct() %>%
    extract_context() %>%
    group_by(folder, service.namespace, env) %>%
    summarise(requests = n())
}
