load_profit <- function(spans) {
  spans %>%
    group_by(folder, service.namespace, metric_group) %>%
    select(
      folder,
      metric_group,
      service.namespace,
      # budget,
      new_budget,
      timestamp,
      cost,
      budget_increment
    ) %>%
    # filter(!is.na(budget)) %>%
    arrange(timestamp) %>%
    summarise(
      # first = first(new_budget, na_rm = TRUE),
      # last = last(new_budget, na_rm = TRUE),
      profit = last(new_budget, na_rm = TRUE) - first(new_budget, na_rm = TRUE),
      spent = sum(cost, na.rm = TRUE),
      gains = sum(budget_increment, na.rm = TRUE),
    ) %>%
    # mutate(gains = profit + spent) %>%
    mutate(roi = profit / spent) %>%
    ungroup() %>%
    filter(!is.na(roi) & !is.infinite(roi))
}
