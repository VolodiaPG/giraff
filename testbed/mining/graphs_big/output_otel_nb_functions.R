big_output_otel_nb_functions_plot <- function(
  functions,
  nb_nodes
) {
  df <- functions %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group) %>%
    summarise(total = sum(n)) %>%
    extract_context() %>%
    left_join(nb_nodes, by = c("folder")) %>%
    extract_env_name() %>%
    env_live_extract()
  # categorize_nb_nodes()
  #
  # df_mean <- df %>%
  #   group_by(env, nb_nodes) %>%
  #   summarise(total = mean(total))

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = total,
      color = env_live,
      fill = env_live
      # group = env
    )
  ) +
    # geom_col(
    #   data = df_mean,
    #   aes(y = total, fill = env),
    #   position = position_dodge(width = 0.9),
    #   alpha = 0.8,
    # ) +
    facet_grid(cols = vars(env)) +
    geom_point(alpha = 0.5) +
    geom_smooth(
      method = "lm",
      se = TRUE,
      fullrange = TRUE,
      level = 0.95,
      alpha = 0.3,
    ) +
    # geom_abline(intercept = 0, slope = 1, linetype = "dashed", color = "red") +
    # geom_beeswarm(
    #   aes(y = total, group = env),
    #   dodge.width = 0.9,
    #   position = position_dodge(width = 0.9),
    #   alpha = 0.5
    # ) +
    # theme(
    #   axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    # ) +
    labs(
      x = "Number of nodes",
      y = "Total number of functions",
      fill = "Flavors",
      color = "Flavors"
    ) +
    guides(group = "none", linetype = "none", alpha = "none") +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
