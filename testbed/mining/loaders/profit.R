load_profit <- function(spans) {
  spans %>%
    group_by(folder, service.namespace, metric_group) %>%
    select(folder, metric_group, service.namespace, budget, timestamp) %>%
    filter(!is.na(budget)) %>%
    arrange(timestamp) %>%
    summarise(profit = last(budget) - first(budget))
}
