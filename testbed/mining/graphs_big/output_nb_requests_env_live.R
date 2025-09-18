big_output_nb_requests_env_live_plot <- function(
  nb_requests,
  nb_nodes
) {
  df <- nb_requests %>%
    group_by(folder, metric_group, service.namespace) %>%
    summarise(requests = sum(requests)) %>%
    extract_context()
  # group_by(folder, env) %>%
  # summarise(requests = mean(requests))

  centered <- df %>%
    group_by(run) %>%
    summarise(mean = mean(requests), dev = sd(requests))

  df <- df %>%
    left_join(centered) %>%
    mutate(requests = (requests - mean) / dev) %>%
    group_by(folder, env) %>%
    summarise(requests = mean(requests)) %>%
    left_join(nb_nodes, by = c("folder")) %>%
    mutate(nb_nodes = factor(nb_nodes)) %>%
    extract_context() %>%
    env_live_extract()

  df_mean <- df %>%
    group_by(env_live) %>%
    summarise(requests = mean(requests))

  ggplot(
    data = df,
    aes(
      x = env_live,
      y = requests,
      group = env_live
    )
  ) +
    geom_col(
      data = df_mean,
      aes(x = env_live, y = requests, fill = env_live),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(
      position = position_dodge(width = 0.9),
      aes = aes(color = env_live, size = nb_nodes),
    ) +
    geom_hline(yintercept = 0, color = "black", linetype = "dashed") +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0)
    ) +
    labs(
      x = "Number of nodes",
      y = "Mean number of requests, centered-reduced"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
