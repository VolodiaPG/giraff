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
    extract_context() %>%
    env_live_extract() %>%
    mutate(env_live = factor(env_live, levels = unique(env_live)))

  df_mean <- df %>%
    group_by(env_live) %>%
    summarise(requests = mean(requests))

  ggplot(
    data = df,
    aes(
      x = env_live,
      y = requests,
    )
  ) +
    geom_col(
      data = df_mean,
      aes(x = env_live, y = requests, group = env_live),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(
      aes(
        size = nb_nodes,
        color = interaction(run, env),
        group = env_live,
      ),
      position = position_dodge(width = 0.9),
    ) +
    geom_line(
      aes(
        group = interaction(run, env),
        color = interaction(run, env),
        x = env_live,
        y = requests,
      ),
      alpha = 0.7,
      linetype = "dotted",
    ) +
    geom_hline(yintercept = 0, color = "black", linetype = "solid") +
    theme(
      legend.position = "none",
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
