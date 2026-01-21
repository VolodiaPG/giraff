load_fallbacks_processed <- function(
  spans,
  errors
) {
  df <- spans %>%
    ungroup() %>%
    filter(span.name == "start_processing_requests") %>%
    mutate(
      otel_error = otel.status_code == "Error"
      # otel_error = if ("otel.status_code" %in% names(df)) {
      #   ifelse(
      #     is.na(otel.status_code),
      #     FALSE,
      #     otel.status_code == "Error"
      #   )
      # } else {
      #   FALSE # Default value if column does not exist
      # }
    ) %>%
    mutate(from = "spmans")

  Log(colnames(df))

  # df <- df %>%
  #   select(
  #     folder,
  #     service.name,
  #     service.namespace,
  #     metric_group,
  #     trace_id,
  #     otel_error,
  #     span.name
  #   ) %>%
  #   distinct()
  Log(colnames(errors))

  df <- df %>%
    full_join(
      errors %>%
        mutate(from = "errors"),
      by = c(
        "folder",
        "metric_group",
        "trace_id"
      )
    )

  Log(colnames(df))

  df <- df %>%
    mutate(
      status = case_when(
        timeout ~ "Failure",
        otel_error ~ "Failure",
        error ~ "Failure",
        fallbacks == 1 ~ "1 fallback",
        fallbacks == 2 ~ "2 fallbacks",
        fallbacks == 0 ~ "0 fallback",
        TRUE ~ NA,
      ),
    )
  df
}
