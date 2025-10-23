big_output_otel_fallbacks_plot <- function(
  spans,
  errors,
  nb_nodes
) {
  # Log(errors %>% select(otel.status_code))

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
        fallbacks == 1 ~ "Succes, Degraded with 1 fallback",
        fallbacks == 2 ~ "Succes, Degraded with 2 fallbacks",
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
          "Succes, Degraded with 1 fallback",
          "Succes, Degraded with 2 fallbacks"
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

  # anova_model <- aov(n ~ env_live + status + env, data = df)
  # tukey_result <- TukeyHSD(anova_model)
  #
  # cld <- multcompLetters4(anova_model, tukey_result)
  # letters <- data.frame(cld$`env_live:env:status`$Letters)

  # Log(letters)

  df_mean <- df %>%
    group_by(env_live, status, alpha) %>%
    summarise(
      max_n = max(n),
      min_n = min(n),
      sd = sd(n, na.rm = TRUE),
      nb = n(),
      n = mean(n)
    )
  # %>%
  # mutate(
  #   se = sd / sqrt(nb),
  #   lower.ci = n - qnorm(0.975) * se,
  #   upper.ci = n + qnorm(0.975) * se
  # ) %>%
  # rowwise() %>%
  # mutate(
  #   lower.ci = max(0, lower.ci),
  # )
  # arrange(desc(n))

  # df_mean$letters <- letters$cld..env_live.env.status..Letters

  df <- df %>%
    group_by(folder, status, env_live, run, alpha) %>%
    summarise(sd = sd(n, na.rm = TRUE), nb = n(), n = mean(n))
  # mutate(se = sd / sqrt(nb), lower.se = n - se, upper.se = n + se)

  ggplot(
    data = df,
    aes(
      x = status,
      y = n,
      group = env_live
    )
  ) +
    # facet_grid(cols = vars(status)) +
    geom_col(
      data = df_mean,
      aes(y = n, fill = env_live),
      position = position_dodge(width = 0.9),
      alpha = 0.8
    ) +
    # geom_errorbar(
    #   data = df_mean,
    #   aes(ymin = lower.ci, ymax = upper.ci, fill = env_live),
    #   position = position_dodge(width = 0.9),
    #   alpha = 0.5,
    #   width = 0.2
    # ) +
    geom_beeswarm(
      # aes(color = env_live),
      position = position_dodge(width = 0.9),
      dodge.width = 0.9,
      cex = 0.8,
      alpha = 0.5
    ) +
    # geom_linerange(
    #   aes(ymin = lower.se, ymax = upper.se, fill = env_live),
    #   position = position_dodge(width = 0.9),
    #   alpha = 0.3,
    #   width = 0.2
    # ) +
    # geom_text(
    #   data = df_mean,
    #   aes(label = letters, group = env_live),
    #   position = position_dodge(width = 0.9),
    #   vjust = -0.5,
    #   size = 5
    # ) +
    # geom_ribbon(
    #   data = df_mean,
    #   aes(ymin = lower.ci, ymax = upper.ci, fill = env_live),
    #   alpha = 0.2
    # ) +
    # geom_line(
    #   aes(group = run),
    #   # position = position_dodge(width = 0.9),
    #   alpha = 0.2
    # ) +
    guides(group = "none", linetype = "none") +
    scale_y_continuous(labels = scales::percent, limits = c(0, 1)) +
    # theme(
    #   axis.text.x = element_blank() # axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    # ) +
    labs(
      x = "Request status",
      y = "Proportion of requests",
      fill = "Application Configuration",
      color = "Application Configuration"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
