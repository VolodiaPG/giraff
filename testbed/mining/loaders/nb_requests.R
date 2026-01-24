load_nb_requests <- function(spans_processed) {
  df_success <- spans_processed %>%
    ungroup() %>%
    filter(status != "Failure") %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(success = sum(n), .groups = "drop")

  df_failures <- spans_processed %>%
    ungroup() %>%
    filter(status == "Failure") %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(failures = sum(n), .groups = "drop")

  df <- df_success %>%
    full_join(
      df_failures,
      by = c("folder", "metric_group", "service.namespace")
    ) %>%
    mutate(success = ifelse(is.na(success), 0, success)) %>%
    mutate(failures = ifelse(is.na(failures), 0, failures)) %>%
    mutate(total = success + failures)

  # total <- nb_requests %>%
  #   group_by(folder, service.namespace) %>%
  #   summarise(total = sum(n))

  df
}
