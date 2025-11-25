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

  Log(letters)

  df_mean <- df %>%
    group_by(env_live) %>%
    summarise(
      profit_per_request = mean(profit_per_request),
    ) %>%
    ungroup() %>%
    arrange(desc(profit_per_request))

  df_mean$letters <- letters$cld.env_live.Letters
  df_mean <- df_mean %>%
    mutate(letters = paste0("\\tiny{", letters, "}"))

  p <- ggplot(
    data = df,
    aes(
      x = env_live,
      y = profit_per_request,
      fill = env_live,
      color = env_live
    )
  ) +
    # facet_grid(cols = vars(env)) +
    geom_col(
      data = df_mean,
      position = position_dodge(width = 0.9),
      alpha = 0.8
      # color = "none"
    ) +
    # facet_grid(~env) +
    # geom_boxplot(alpha = 0.7) +
    geom_beeswarm(
      # aes(color = env_live),
      alpha = 0.7,
      color = "black",
      position = position_dodge(width = 0.9),
    ) +
    geom_text(
      data = df_mean,
      aes(
        label = letters,
        group = env_live,
        y = max(df$profit_per_request) + 1
      ),
      color = "black",
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
    scale_y_continuous(limits = c(0, max(df$profit_per_request) + 2)) +
    labs(
      x = "Environment Configuration",
      y = "RoI",
      # title = "Profit per Request by Environment Configuration",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    theme(
      # legend.position = "none",
      axis.text.x = element_blank() # axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)

  return(p)
}
