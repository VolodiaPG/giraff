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
          # "Total",
          # "Failure",
          "0 fallback",
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

  df_anova <- df %>%
    select(env_live, env, status, n) %>%
    mutate(n = ifelse(is.na(n), 0, n))

  anova_model <- aov(n ~ env_live * env * status, data = df_anova)
  tukey_result <- TukeyHSD(anova_model)

  cld <- multcompLetters4(anova_model, tukey_result)
  letters <- data.frame(cld$`env_live:env:status`$Letters)

  df_mean <- df %>%
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
          c("0 fallback") |
          env_live %in% keep_letters_for,
        letters,
        ""
      )
    ) %>%
    mutate(letters = paste0("\\tiny{", letters, "}"))

  df_cumsum <- df_mean %>%
    mutate(n = ifelse(is.na(n), 0, n)) %>%
    group_by(env_live, env) %>%
    arrange(status) %>%
    mutate(n_cumsum = cumsum(n))

  df <- df %>%
    group_by(folder, status, env_live, env, run) %>%
    summarise(n = mean(n))

  ggplot(
    data = df,
    aes(
      x = status,
      y = n
    )
  ) +
    facet_grid(cols = vars(env)) +
    geom_col(
      data = df_mean,
      aes(y = n, fill = env_live),
      position = position_dodge(width = 0.9)
    ) +
    geom_beeswarm(
      aes(group = env_live),
      position = position_dodge(width = 0.9),
      dodge.width = 0.9,
      cex = 0.8,
      alpha = 0.5
    ) +
    geom_text(
      data = df_mean,
      aes(label = letters, group = env_live, y = 0.83),
      position = position_dodge(width = 0.9),
      vjust = -0.5,
      size = 5
    ) +
    geom_line(
      data = df_cumsum,
      aes(y = n_cumsum, group = env_live, color = env_live),
      # position = position_dodge(width = 0.9),
      linewidth = 1
    ) +
    geom_point(
      data = df_cumsum,
      aes(y = n_cumsum, group = env_live, color = env_live),
      # position = position_dodge(width = 0.9),
      size = 2
    ) +
    guides(color = "none", linetype = "none") +
    # scale_x_discrete(guide = guide_axis(n.dodge = 2)) +
    scale_y_continuous(labels = scales::percent, limits = c(0, .85)) +
    labs(
      y = "Share of successful responses",
      x = "Number of fallbacks",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    # Fix colors since only two flavors are displayed
    scale_fill_manual(
      values = c("#440154", "#31688e", "#35b7794D", "#fde7254D")
    ) +
    scale_color_manual(
      values = c("#440154", "#31688e", "#35b7794D", "#fde7254D")
    )
}
