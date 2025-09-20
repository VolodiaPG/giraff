output_requests_profit_plot <- function(
  spans,
  nb_requests,
  nb_nodes,
  nb_functions
) {
  df <- spans %>%
    group_by(folder, service.namespace, metric_group) %>%
    select(folder, metric_group, service.namespace, budget, timestamp) %>%
    filter(!is.na(budget)) %>%
    arrange(timestamp) %>%
    summarise(profit = last(budget) - first(budget)) %>%
    left_join(
      nb_requests %>%
        mutate(requests = success) %>%
        select(folder, requests)
    ) %>%
    left_join(nb_functions)

  # mutate(benefit = ifelse(profit >= 0, "Profit", "Loss")) %>%
  # left_join(nb_functions)
  # mutate(profit = profit / requests)
  # mutate(profit = profit / requests / nb_functions)
  # mutate(profit = profit)

  center <- df %>%
    group_by(folder) %>%
    summarise(
      mean = mean(profit),
      sd = sd(profit),
      mean_requests = mean(requests),
      sd_requests = sd(requests),
      mean_nb_functions = mean(nb_functions),
      sd_nb_functions = sd(nb_functions),
      nb = n()
    ) %>%
    mutate(
      se = sd / sqrt(nb),
      lower.ci = mean - qnorm(0.975) * se,
      upper.ci = mean + qnorm(0.975) * se
    )

  df <- df %>%
    left_join(center) %>%
    mutate(
      ci = case_when(
        profit < lower.ci ~ "below",
        profit > upper.ci ~ "above",
        TRUE ~ "within"
      ),
      ci = factor(ci, levels = c("below", "within", "above"))
    ) %>%
    # mutate(profit = (profit - mean) / sd) %>%
    # mutate(requests = (requests - mean_requests) / sd_requests) %>%
    # mutate(
    #   nb_functions = (nb_functions - mean_nb_functions) / sd_nb_functions
    # ) %>%
    left_join(nb_nodes) %>%
    extract_context() %>%
    env_live_extract() %>%
    extract_env_name() %>%
    group_by(run, nb_nodes, folder, env_live, ci, env) %>%
    summarise(
      profit = mean(profit),
      requests = mean(requests),
      nb_functions = mean(nb_functions)
    )

  df_center <- df %>%
    group_by(run) %>%
    summarise(
      mean = mean(profit),
      sd = sd(profit),
      mean_requests = mean(requests),
      sd_requests = sd(requests),
      mean_nb_functions = mean(nb_functions),
      sd_nb_functions = sd(nb_functions),
    )

  df <- df %>%
    left_join(df_center) %>%
    mutate(
      profit = (profit - mean) / sd,
      requests = (requests - mean_requests) / sd_requests,
      nb_functions = (nb_functions - mean_nb_functions) / sd_nb_functions
    ) %>%
    group_by(run, nb_nodes, env_live, ci, env) %>%
    summarise(
      profit = mean(profit),
      requests = mean(requests),
      nb_functions = mean(nb_functions)
    )

  ggplot(
    data = df,
    aes(
      y = profit,
      x = requests,
      size = nb_functions,
      color = env_live,
      fill = env_live,
      # group = folder
    ),
  ) +
    facet_grid(env ~ ci) +
    stat_ellipse(type = "norm", geom = "polygon", alpha = .1) +
    geom_point(alpha = 0.8) +
    # geom_smooth(method = "lm") +
    # facet_grid(benefit ~ env_live) +
    # geom_boxplot(position = position_dodge2()) +
    # geom_hline(yintercept = 0, color = "black", linetype = "dashed") +
    # labs(
    #   x = "Number of nodes",
    #   y = "Centered-reduced profit"
    # ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10)
      # axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
