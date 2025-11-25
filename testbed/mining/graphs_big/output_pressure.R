big_output_pressure_plot <- function(
  nb_requests,
  nb_nodes
) {
  df <- nb_requests %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(requests = sum(success) / sum(requests), .groups = "drop") %>%
    group_by(folder, metric_group) %>%
    summarise(requests = mean(requests), .groups = "drop") %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    env_live_extract() %>%
    categorize_nb_nodes() %>%
    extract_env_name()

  anova_model <- aov(requests ~ env_live + run, data = df)
  tukey_result <- TukeyHSD(anova_model)

  cld <- multcompLetters4(anova_model, tukey_result)
  letters <- data.frame(cld$`env_live`$Letters)

  Log(letters)

  df_mean <- df %>%
    group_by(env_live) %>%
    summarise(requests = mean(requests)) %>%
    arrange(desc(requests))

  df_mean$letters <- letters$cld.env_live.Letters
  df_mean <- df_mean %>%
    mutate(letters = paste0("\\tiny{", letters, "}"))

  ggplot(
    data = df,
    aes(
      x = env_live,
      y = requests,
    )
  ) +
    # facet_grid(cols = vars(env)) +
    geom_col(
      data = df_mean,
      aes(fill = env_live),
      alpha = 0.8,
      position = position_dodge(width = 0.9)
    ) +
    geom_beeswarm(
      # aes(color = env_live),
      position = position_dodge(width = 0.9),
      alpha = 0.5
    ) +
    # annotate(
    #   "text",
    #   y = max(df$requests) + .05,
    #   label = df_mean$letters,
    #   group = df_mean$env_live,
    #   color = "black",
    #   size = 5
    # ) +
    geom_text(
      data = df_mean,
      aes(label = letters, group = env_live, y = max(df$requests) + 0.02),
      position = position_dodge(width = 0.9),
      vjust = -0.5,
      size = 5
    ) +
    theme(axis.text.x = element_blank()) +
    scale_y_continuous(labels = scales::percent, limits = c(0, 1.05)) +
    guides(group = "none") +
    labs(
      x = "Number of nodes",
      y = "Number of successful requests",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
