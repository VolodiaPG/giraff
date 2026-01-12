big_output_nb_requests_env_live_plot <- function(
  profit,
  nb_requests,
  nb_nodes
) {
  # Log(colnames(nb_requests))
  # df <- nb_requests %>%
  #   # group_by(folder, metric_group, service.namespace) %>%
  #   # summarise(requests = sum(success)) %>%
  #   mutate(requests = success) %>%
  #   extract_context()
  # # group_by(folder, env) %>%
  # # summarise(requests = mean(requests))
  #
  # centered <- df %>%
  #   group_by(run, env) %>%
  #   summarise(mean = mean(requests), dev = sd(requests))
  #
  # df <- df %>%
  #   left_join(centered) %>%
  #   mutate(requests = (requests - mean) / dev) %>%
  #   group_by(folder, env) %>%
  #   summarise(requests = mean(requests)) %>%
  #   left_join(nb_nodes, by = c("folder")) %>%
  #   extract_context() %>%
  #   env_live_extract() %>%
  #   mutate(env_live = factor(env_live, levels = unique(env_live))) %>%
  #   extract_env_name()
  #
  # df_mean <- df %>%
  #   group_by(env_live) %>%
  #   summarise(requests = mean(requests))

  df <- profit %>%
    # left_join(profit) %>%
    left_join(nb_nodes) %>%
    extract_context() %>%
    env_live_extract() %>%
    extract_env_name() %>%
    categorize_nb_nodes() %>%
    mutate(benefit = profit > 0) %>%
    group_by(env_live, env, folder, nb_nodes) %>%
    summarise(nb_benefit = sum(benefit), nb = n()) %>%
    mutate(ratio_benefit = nb_benefit / nb)

  anova_model <- aov(ratio_benefit ~ env_live, data = df)
  tukey_result <- TukeyHSD(anova_model)

  # Log(tukey_result)

  cld <- multcompLetters4(anova_model, tukey_result)
  letters <- data.frame(cld$`env_live`$Letters)
  # cld_df <- data.frame(
  #   env_live = names(cld$`env_live:env`$Letters),
  #   letter = cld$`env_live:env`$Letters
  # )

  df_mean <- df %>%
    group_by(env_live) %>%
    summarise(
      nb_benefit = mean(nb_benefit),
      ratio_benefit = mean(ratio_benefit)
    ) %>%
    ungroup() %>%
    arrange(desc(ratio_benefit))

  df_mean$letters <- letters$cld.env_live.Letters
  df_mean <- df_mean %>%
    mutate(letters = paste0("\\tiny{", letters, "}"))
  # cld_df <- cld_df %>%
  #   separate(env_live, into = c("env_live", "env"), sep = ":")
  #
  # Log(cld_df)

  # df_mean_letters <- df_mean %>%
  #   left_join(cld_df, by = c("env_live", "env"))

  Log(df_mean)

  ggplot(
    data = df,
    aes(
      x = env_live,
      y = ratio_benefit,
      # group = env_live
    )
  ) +
    # facet_wrap(~env) +
    geom_col(
      data = df_mean,
      aes(fill = env_live),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_beeswarm(
      aes(
        # size = nb_nodes,
        # color = env_live,
      ),
      dodge.width = 0.9,
      alpha = 0.5,
      position = position_dodge(width = 0.9),
    ) +
    geom_text(
      data = df_mean,
      aes(label = letters, group = env_live, y = max(df$ratio_benefit) + 0.02),
      position = position_dodge(width = 0.9),
      vjust = -0.5,
      size = 5
    ) +
    theme(
      #   axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
      axis.text.x = element_blank()
    ) +
    labs(
      x = "Flavors",
      y = "Share of profitable applications",
      fill = APP_CONFIG,
      color = APP_CONFIG
    ) +
    scale_y_continuous(
      labels = scales::percent,
      limits = c(0, max(df$ratio_benefit) + 0.05)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
