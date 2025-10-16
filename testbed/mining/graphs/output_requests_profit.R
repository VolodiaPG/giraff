output_requests_profit_plot <- function(
  profit,
  nb_requests,
  nb_nodes,
  nb_functions
) {
  df <- profit %>%
    left_join(
      nb_requests %>%
        mutate(requests = requests) %>%
        select(folder, requests)
    ) %>%
    left_join(nb_functions) %>%
    left_join(nb_nodes) %>%
    extract_context() %>%
    mutate(env_live_raw = env_live) %>%
    env_live_extract() %>%
    extract_env_name() %>%
    mutate(profit_per_request = roi) %>%
    group_by(run, nb_nodes, folder, env_live, env_live_raw, env) %>%
    summarise(
      profit_per_request = mean(profit_per_request),
      .groups = "drop"
    )

  anova_model <- aov(profit_per_request ~ env_live, data = df)
  tukey_result <- TukeyHSD(anova_model)

  cld <- multcompLetters4(anova_model, tukey_result)
  letters <- data.frame(cld$`env_live`$Letters)

  df_mean <- df %>%
    group_by(env_live) %>%
    summarise(
      profit_per_request = mean(profit_per_request),
    ) %>%
    arrange(desc(profit_per_request))

  df_mean$letters <- letters$cld.env_live.Letters

  Log(df_mean)

  p <- ggplot(
    data = df,
    aes(
      x = env_live,
      y = profit_per_request,
      fill = env_live
    )
  ) +
    geom_col(
      data = df_mean,
      aes(fill = env_live),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    # facet_grid(~env) +
    # geom_boxplot(alpha = 0.7) +
    geom_point(
      aes(
        shape = run
      ),
      alpha = 0.7,
      position = position_dodge(width = 0.9),
    ) +
    geom_text(
      data = df_mean,
      aes(label = letters, group = env_live),
      position = position_dodge(width = 0.9),
      vjust = -0.5,
      size = 5
    ) +
    # stat_pvalue_manual(
    #   pwc,
    #   hide.ns = TRUE,
    #   tip.length = 0,
    #   # step.increase = 0.1,
    #   # step.group.by = "env_live"
    # ) +
    # ggpubr::stat_pvalue_manual(
    #   pwc,
    #   hide.ns = TRUE,
    #   tip.length = 0.01
    # ) +
    labs(
      x = "Environment Configuration",
      y = "Profit per Request",
      title = "Profit per Request by Environment Configuration",
      fill = "Application Configuration",
      color = "Application Configuration"
    ) +
    theme_minimal() +
    theme(legend.position = "none") +
    scale_fill_viridis_d() +
    guides(shape = "none")

  return(p)
}
