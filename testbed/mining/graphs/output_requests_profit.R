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
    mutate(benefit = ifelse(profit >= 0, "Profit", "Loss")) %>%
    left_join(nb_requests) %>%
    left_join(nb_functions) %>%
    mutate(profit = profit / requests / nb_functions)

  center <- df %>%
    group_by(folder) %>%
    summarise(mean = mean(profit), dev = sd(profit))

  df <- df %>%
    left_join(center) %>%
    mutate(profit = (profit - mean) / dev) %>%
    left_join(nb_nodes) %>%
    mutate(nb_nodes = factor(nb_nodes)) %>%
    extract_context() %>%
    env_live_extract()

  ggplot(
    data = df,
    aes(
      x = nb_nodes,
      y = profit,
      fill = benefit,
      group = folder
    ),
  ) +
    facet_grid(benefit ~ env_live) +
    geom_boxplot(position = position_dodge2()) +
    geom_hline(yintercept = 0, color = "black", linetype = "dashed") +
    labs(
      x = "Number of nodes",
      y = "Centered-reduced profit"
    ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
