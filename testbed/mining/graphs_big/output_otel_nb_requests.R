big_output_otel_nb_requests_plot <- function(
  nb_requests,
  nb_nodes
) {
  df <- nb_requests %>%
    group_by(folder, metric_group) %>%
    summarise(requests = sum(total), success = sum(success)) %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    extract_env_name() %>%
    env_live_extract()
  # categorize_nb_nodes()

  # df_mean <- df %>%
  #   group_by(env, nb_nodes, env_live) %>%
  #   summarise(requests = mean(requests))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = requests,
      color = env_live,
      fill = env_live
    )
  ) +
    # facet_grid(cols = vars(env_live)) +
    geom_point(alpha = 0.3) +
    geom_smooth(
      method = "lm",
      se = TRUE,
      fullrange = TRUE,
      level = 0.95,
      alpha = 0.3,
    ) +
    # geom_smooth(
    #   aes(y = success),
    #   method = "lm",
    #   se = TRUE,
    #   fullrange = TRUE,
    #   level = 0.95,
    #   alpha = 0.5,
    #   linetype = "dotted"
    # ) +
    # geom_col(
    #   data = df_mean,
    #   aes(y = requests, fill = env),
    #   position = position_dodge(width = 0.9),
    #   alpha = 0.8,
    # ) +
    # geom_point(
    #   aes(y = requests, color = env),
    #   position = position_dodge(width = 0.9)
    # ) +
    theme(
      axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    ) +
    labs(
      x = "Number of nodes",
      y = "Number of requests",
      fill = APP_CONFIG,
      color = APP_CONFIG,
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
