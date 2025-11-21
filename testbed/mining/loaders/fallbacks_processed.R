load_fallbacks_processed <- function(
  spans,
  errors,
  nb_nodes
) {
  requests <- spans %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    mutate(
      otel_error = if ("otel.status_code" %in% names(df)) {
        ifelse(
          is.na(otel.status_code),
          FALSE,
          otel.status_code == "Error"
        )
      } else {
        FALSE # Default value if column does not exist
      }
    ) %>%
    #
    #
    # mutate(
    #   otel_error = ifelse(
    #     is.na(otel.status_code),
    #     FALSE,
    #     otel.status_code == "Error"
    #   )
    # ) %>%
    select(folder, service.namespace, metric_group, trace_id, otel_error) %>%
    distinct()
  #
  # Log(
  #   requests %>%
  #     select(folder, service.namespace, trace_id) %>%
  #     group_by(folder, service.namespace) %>%
  #     summarise(total = n())
  # )
  df <- requests %>%
    # left_join(degrades) %>%
    left_join(
      errors
    ) %>%
    mutate(
      status = case_when(
        timeout ~ "Failure",
        otel_error ~ "Failure",
        error ~ "Failure",
        fallbacks == 1 ~ "Success, Degraded with 1 fallback",
        fallbacks == 2 ~ "Success, Degraded with 2 fallbacks",
        TRUE ~ "Success, Nominal",
      ),
    ) %>%
    filter(!is.na(status)) %>%
    group_by(
      folder,
      metric_group,
      service.namespace,
      status
    ) %>%
    summarise(n = n(), .groups = "drop") %>%
    left_join(
      requests %>%
        select(folder, service.namespace, trace_id) %>%
        group_by(folder, service.namespace) %>%
        summarise(total = n())
    )

  total_successes <- df %>%
    filter(status != "Failure" & status != "Timeout") %>%
    group_by(folder, metric_group, service.namespace, total) %>%
    summarise(n = sum(n)) %>%
    mutate(status = "Total Successes")

  df <- df %>%
    bind_rows(total_successes) %>%
    mutate(n = n / total) %>%
    filter(!is.na(n)) %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    env_live_extract() %>%
    extract_env_name() %>%
    categorize_nb_nodes() %>%
    mutate(
      status = factor(
        status,
        levels = c(
          "Failure",
          "Total Successes",
          "Success, Nominal",
          "Success, Degraded with 1 fallback",
          "Success, Degraded with 2 fallbacks"
        )
      )
    ) %>%
    mutate(
      alpha = case_when(
        status == "Failure" ~ 1,
        status == "Total Successes" ~ 1,
        TRUE ~ 0.8
      )
    )

  df
}
