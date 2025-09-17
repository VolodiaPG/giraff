load_nb_requests <- function(spans) {
  spans %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    select(folder, service.namespace, metric_group, trace_id) %>%
    distinct() %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(requests = n())
}
