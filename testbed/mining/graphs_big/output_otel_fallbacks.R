big_output_otel_fallbacks_plot <- function(
  fallbacks_processed
) {
  df <- fallbacks_processed %>%
    filter(env_live %in% c(SCE_ONE, SCE_TWO)) %>%
    mutate(
      env_live = factor(env_live, levels = c(SCE_ONE, SCE_TWO))
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
      max_n = max(n),
      min_n = min(n),
      sd = sd(n, na.rm = TRUE),
      nb = n(),
      n = mean(n)
    ) %>%
    arrange(desc(n))

  df_mean$letters <- letters$cld..env_live.env.status..Letters

  df_mean <- df_mean %>%
    mutate(letters = ifelse(str_length(letters) > 5, "", letters)) %>%
    mutate(letters = paste0("\\tiny{", letters, "}"))

  df <- df %>%
    group_by(folder, status, env_live, env, run) %>%
    summarise(sd = sd(n, na.rm = TRUE), nb = n(), n = mean(n))
  # mutate(se = sd / sqrt(nb), lower.se = n - se, upper.se = n + se)

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
      position = position_dodge(width = 0.9),
      alpha = 0.8
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
      aes(label = letters, group = env_live, y = 1),
      position = position_dodge(width = 0.9),
      vjust = -0.5,
      size = 5
    ) +
    guides(color = "none", linetype = "none") +
    scale_x_discrete(guide = guide_axis(n.dodge = 2)) +
    scale_y_continuous(labels = scales::percent, limits = c(0, 1.05)) +
    labs(
      y = "Share of successful responses",
      x = "Number of fallbacks",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    # Fix colors since only two flavors are displayed
    scale_fill_manual(values = c("#440154", "#31688e")) +
    scale_color_manual(values = c("#440154", "#31688e"))
}
