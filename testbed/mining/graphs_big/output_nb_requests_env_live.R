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

  df_mean <- df %>%
    group_by(env_live, env) %>%
    summarise(
      nb_benefit = mean(nb_benefit),
      ratio_benefit = mean(ratio_benefit)
    )

  ggplot(
    data = df,
    aes(
      x = env,
      y = ratio_benefit,
      # shape = env,
      group = env_live
    )
  ) +
    # facet_wrap(~env) +
    geom_col(
      data = df_mean,
      aes(fill = env_live),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(
      aes(
        # size = nb_nodes,
        color = env_live,
      ),
      position = position_dodge(width = 0.9),
    ) +
    theme(
      axis.text.x = element_text(angle = 45, vjust = 1, hjust = 1)
    ) +
    labs(
      x = "Number of nodes",
      y = "Ratio of functions that made a profit"
    ) +
    scale_y_continuous(labels = scales::percent) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
