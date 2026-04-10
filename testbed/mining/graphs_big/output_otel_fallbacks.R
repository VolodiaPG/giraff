big_output_otel_fallbacks_plot <- function(
  fallbacks_processed,
  nb_requests
) {
  env_live_levels <- c(SCE_ONE, SCE_TWO, SCE_THREE)
  keep_letters_for <- c(SCE_TWO, SCE_THREE, SCE_FOUR)

  fallbacks_processed <- fallbacks_processed %>%
    filter(env_live %in% env_live_levels) %>%
    filter(status != "Failure") %>%
    mutate(
      status = factor(
        status,
        levels = c(
          # "Failure",
          "0 fallbacks",
          "1 fallback",
          "2 fallbacks"
        )
      )
    )

  total <- nb_requests %>%
    group_by(folder) %>%
    summarise(nb_functions = n())

  df <- fallbacks_processed %>%
    left_join(nb_requests, by = c("folder", "service.namespace")) %>%
    mutate(n = n / total) %>%
    group_by(folder, status, env, env_live, run) %>%
    summarise(n = sum(n)) %>%
    left_join(total, by = c("folder")) %>%
    mutate(n = n / nb_functions) %>%
    mutate(
      env_live = factor(env_live, levels = env_live_levels)
    ) %>%
    ungroup() %>%
    complete(status, env_live, env, fill = list(n = NA))

  # Individual points for Total column
  df_total_points <- df %>%
    filter(!is.na(n)) %>%
    select(folder, env_live, env, run, n) %>%
    group_by(folder, env_live, env, run) %>%
    summarise(n = sum(n), .groups = "drop") %>%
    mutate(
      status = "Total",
      plot_group = "Total"
    )

  df_anova <- df %>%
    bind_rows(df_total_points) %>%
    select(env_live, env, status, n) %>%
    mutate(n = ifelse(is.na(n), 0, n))

  anova_model <- aov(n ~ env_live * env * status, data = df_anova)
  tukey_result <- TukeyHSD(anova_model)

  cld <- multcompLetters4(anova_model, tukey_result)
  letters <- data.frame(
    cld$`env_live:env:status`$Letters,
    stringsAsFactors = FALSE
  )

  df_mean <- df_anova %>%
    group_by(env_live, env, status) %>%
    summarise(
      n = mean(n)
    ) %>%
    arrange(desc(n))

  df_mean$letters <- letters$cld..env_live.env.status..Letters

  df_mean <- df_mean %>%
    mutate(
      letters = ifelse(
        status %in%
          c("0 fallbacks", "Total") |
          env_live %in% keep_letters_for,
        letters,
        ""
      )
    ) %>%
    mutate(letters = paste0("\\tiny{", letters, "}"))

  # Add plot_group for faceting: Total separate from details
  df_mean <- df_mean %>%
    mutate(
      plot_group = ifelse(
        status == "Total",
        "Total",
        "Details"
      )
    )

  df_total <- df_mean %>%
    group_by(env_live, env) %>%
    summarise(n = sum(n, na.rm = TRUE), .groups = "drop") %>%
    mutate(
      status = "Total",
      plot_group = "Total"
    )

  df <- df %>%
    filter(status != "Total") %>%
    group_by(folder, status, env_live, env, run) %>%
    summarise(n = mean(n)) %>%
    mutate(plot_group = "Details")

  ggplot(
    data = df,
    aes(
      x = status,
      y = n
    )
  ) +
    facet_grid(
      cols = vars(env, plot_group),
      scales = "free_x",
      space = "free_x",
      shrink = TRUE,
      drop = TRUE,
      labeller = labeller(plot_group = function(x) rep("", length(x)))
    ) +
    geom_col(
      data = df_mean %>% filter(status == "Total"),
      aes(y = n, color = env_live),
      fill = "transparent",
      position = position_dodge(width = 1),
      width = 0.7,
      alpha = 0.9
    ) +
    geom_beeswarm(
      data = df_total_points,
      aes(group = env_live),
      position = position_dodge(width = 1),
      dodge.width = 1,
      alpha = 0.5
    ) +
    geom_col(
      data = df_mean %>% filter(status != "Total"),
      aes(y = n, fill = env_live),
      position = position_dodge(width = 1),
      alpha = 0.8
    ) +
    geom_beeswarm(
      aes(group = env_live),
      position = position_dodge(width = 1),
      dodge.width = 1,
      alpha = 0.5
    ) +
    geom_text(
      data = df_mean,
      aes(label = letters, group = env_live, y = 0.83),
      position = position_dodge(width = 1),
      vjust = -0.5,
      size = 5
    ) +
    guides(group = "none", linetype = "none", alpha = "none", fill = "none") +
    scale_x_discrete(
      guide = guide_prism_bracket(width = 0.15),
      labels = scales::wrap_format(5)
    ) +
    scale_y_continuous(labels = scales::percent, limits = c(0, .85)) +
    labs(
      y = "Share of successful responses",
      x = "Number of fallbacks involved in the response",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    # Fix colors since only two flavors are displayed
    scale_fill_manual(
      values = c("#440154", "#31688e", "#35b779", "#fff"),
      guide = guide_legend(override.aes = list(size = 2, alpha = 1))
    ) +
    scale_color_manual(
      values = c("#440154", "#31688e", "#35b779", "#fde725"),
      guide = guide_legend(override.aes = list(size = 2, alpha = 1))
    )
}
