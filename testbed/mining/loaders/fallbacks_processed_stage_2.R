load_fallbacks_processed_stage2 <- function(
  fallbacks_processed
) {
  df <- fallbacks_processed %>%
    filter(!is.na(status))

  # tmp <- df %>%
  #   # filter(service.namespace == "6e255c75-3104-4433-88d6-687020aaaaf5") %>%l
  #   filter(is.na(status)) %>%
  #   sample_n(10) %>%
  #   full_join(
  #     df %>%
  #       filter(!is.na(status) & from.y != "error") %>%
  #       sample_n(10)
  #   )
  #
  # Log(
  #   df %>%
  #     select(folder, trace_id) %>%
  #     group_by(folder, trace_id) %>%
  #     summarise(n = n()) %>%
  #     filter(n > 1)
  # )
  #
  # write_csv(tmp, "tmp.csv")

  # total <- df %>%
  #   group_by(
  #     folder,
  #     metric_group,
  #     service.namespace
  #   ) %>%
  #   summarise(total = n(), .groups = "drop")

  df <- df %>%
    group_by(
      folder,
      metric_group,
      service.namespace,
      status
    ) %>%
    summarise(n = n(), .groups = "drop")

  # total_successes <- df %>%
  #   filter(status != "Failure") %>%
  #   group_by(folder, metric_group, service.namespace) %>%
  #   summarise(n = sum(n)) %>%
  #   mutate(status = "Total")

  df <- df %>%
    # bind_rows(total_successes) %>%
    # left_join(total) %>%
    extract_context() %>%
    env_live_extract() %>%
    extract_env_name() %>%
    # filter(status != "Failure") %>%
    mutate(
      status = factor(
        status,
        levels = c(
          "Failure",
          # "Total",
          "0 fallback",
          "1 fallback",
          "2 fallbacks"
        )
      )
    )
  df
}
