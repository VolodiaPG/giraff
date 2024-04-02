output_provisioned_simple <- memoised(function(functions_total) {
  df <- functions_total %>% filter(status == "provisioned")
  provisioned <- df %>%
    ggplot(aes(x = metric_group, y = ratio)) +
    # geom_quasirandom(method = "tukey", alpha = .2) +$
    facet_grid(rows = vars(docker_fn_name)) +
    labs(
      x = "Function",
      y = "Number function provisioned"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    geom_point(aes(color = metric_group, fill = metric_group, )) +
    geom_line(aes(group = metric_group_group), alpha = .2) +
    stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)

  return(provisioned)
})

output_provisioned_simple_total <- memoised(function(functions_total) {
  df <- functions_total %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group_group, metric_group) %>%
    summarise(n = sum(n), total = sum(total)) %>%
    mutate(ratio = n / total)


  provisioned <- df %>%
    ggplot(aes(x = metric_group, y = ratio)) +
    # geom_quasirandom(method = "tukey", alpha = .2) +$
    labs(
      x = "Function",
      y = "Number function provisioned"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    geom_point(aes(color = metric_group, fill = metric_group, )) +
    geom_line(aes(group = metric_group_group), alpha = .2) +
    stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)

  return(provisioned)
})

output_jains_simple <- memoised(function(earnings) {
  jains <- earnings %>%
    ggplot(aes(alpha = 1, x = metric_group, y = score)) +
    geom_hline(yintercept = max(earnings$worst_case), color = "black") +
    # geom_quasirandom(method='tukey',alpha=.2)+
    labs(
      x = "Placement method",
      y = "Jain's index"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    geom_point(aes(color = metric_group, fill = metric_group, )) +
    geom_line(aes(group = metric_group_group), alpha = .2) +
    stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)

  return(jains)
})


output_spending_plot_simple <- memoised(function(plots.spending.data) {
  df <- plots.spending.data %>%
    extract_function_name_info()

  p <- ggplot(data = df, aes(x = winner, y = cost, color = docker_fn_name, alpha = 1)) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Function cost",
      x = "Placement method",
    ) +
    theme(legend.background = element_rect(
      fill = alpha("white", .7),
      size = 0.2, color = alpha("white", .7)
    )) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    guides(colour = guide_legend(nrow = 1)) +
    geom_quasirandom(method = "tukey", alpha = .2)

  return(p)
})

output_number_requests <- memoised(function(respected_sla) {
  df <- respected_sla %>%
    group_by(folder, metric_group, metric_group_group, docker_fn_name) %>%
    summarise(total = sum(total))

  p <- ggplot(data = df, aes(x = metric_group, y = total, alpha = 1)) +
    theme(legend.position = "none") +
    facet_grid(rows = vars(docker_fn_name)) +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "total requests",
      x = "function name",
    ) +
    theme(legend.background = element_rect(
      fill = alpha("white", .7),
      size = 0.2, color = alpha("white", .7)
    )) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    guides(colour = guide_legend(nrow = 1)) +
    geom_point(aes(color = metric_group, fill = metric_group, )) +
    geom_line(aes(group = metric_group_group), alpha = .2) +
    stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)

  return(p)
})

output_respected_data_plot_total <- memoised(function(respected_sla) {
  df <- respected_sla %>%
    group_by(metric_group, metric_group_group, folder) %>%
    filter(docker_fn_name == "echo") %>%
    select(metric_group_group, metric_group, folder, acceptable_chained) %>%
    summarise(acceptable_chained = sum(acceptable_chained)) %>%
    inner_join(respected_sla %>%
      group_by(metric_group, metric_group_group, folder) %>%
      filter(prev_function == "<iot_emulation>") %>%
      select(metric_group_group, metric_group, folder, total) %>%
      summarise(total = sum(total))) %>%
    mutate(satisfied_ratio = acceptable_chained / total) %>%
    ungroup()

  p <- ggplot(data = df, aes(alpha = 1, x = metric_group, y = satisfied_ratio)) +
    theme(legend.position = "none") +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    scale_y_continuous(labels = scales::percent) +
    labs(
      x = "memtric_group",
      y = "satisfied_ratio"
    ) +
    geom_beeswarm(aes(color = metric_group, fill = metric_group, )) +
    geom_line(aes(group = metric_group_group), alpha = .2)
  #  stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)
  return(p)
})

output_number_requests_total <- memoised(function(respected_sla) {
  df <- respected_sla %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(total = sum(total))

  p <- ggplot(data = df, aes(x = metric_group, y = total, alpha = 1)) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "total requests",
      x = "function name",
    ) +
    theme(legend.background = element_rect(
      fill = alpha("white", .7),
      size = 0.2, color = alpha("white", .7)
    )) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    guides(colour = guide_legend(nrow = 1)) +
    geom_point(aes(color = metric_group, fill = metric_group, )) +
    geom_line(aes(group = metric_group_group), alpha = .2) +
    stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)

  return(p)
})
