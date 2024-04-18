output_provisioned_simple <- function(functions_total) {
  df <- functions_total %>% filter(status == "provisioned")
  provisioned <- df %>%
    ggplot(aes(x = metric_group, y = n)) +
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
}

output_provisioned_simple_total <- function(functions_total) {
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
}

output_jains_simple <- function(earnings, functions_all_total, node_levels) {
  df <- earnings %>%
    inner_join(
      functions_all_total,
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    # inner_join(node_levels %>% group_by(metric_group, metric_group_group, folder) %>% summarise(nodes = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    extract_context() %>%
    mutate(total2 = total / n)

  jains <- df %>%
    ggplot(aes(alpha = 1, x = n, y = score)) +
    # facet_grid(cols = vars(n)) +
    geom_hline(yintercept = max(earnings$worst_case), color = "black") +
    labs(
      x = "number of fog nodes",
      y = "Jain's index"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    geom_point(aes(shape = run, size = env, color = placement_method, fill = placement_method))
  # geom_quasirandom(aes(shape = env, color = placement_method), method = "tukey", alpha = .4) +
  # geom_line(aes(group = env), alpha = .2)
  # stat_summary(aes(color = placement_method, fill = placement_method), fun = mean, geom = "bar", position = "dodge", alpha = 0.5)

  return(jains)
}


output_spending_plot_simple_total <- function(bids_won, node_levels) {
  df <- bids_won %>%
    extract_function_name_info() %>%
    left_join(node_levels %>% rename(winner = name)) %>%
    # group_by(folder, metric_group, metric_group_group, level_value) %>%
    # summarise(cost = mean(cost)) %>%
    extract_context()

  p <- ggplot(data = df, aes(x = env, y = cost, alpha = 1)) +
    theme(legend.position = "none") +
    facet_grid(rows = vars(level_value), cols = vars(placement_method)) +
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
    geom_quasirandom(aes(shape = run, color = env), method = "tukey", alpha = .2) +
    # geom_line(aes(group = metric_group_group), alpha = .2) +
    # geom_point(aes(color = metric_group, fill = metric_group, )) +
    # stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)
    geom_boxplot()

  return(p)
}

output_number_requests <- function(respected_sla, node_levels) {
  df <- respected_sla %>%
    group_by(folder, metric_group, metric_group_group, docker_fn_name) %>%
    summarise(total = sum(total)) %>%
    inner_join(node_levels %>% group_by(metric_group, metric_group_group, folder) %>% summarise(nodes = n()), by = c("folder", "metric_group", "metric_group_group"))

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
}

output_respected_data_plot_total <- function(respected_sla, node_levels) {
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
    extract_context() %>%
    inner_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(n = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    ungroup()

  p <- ggplot(data = df, aes(alpha = 1, x = satisfied_ratio, y = total)) +
    facet_grid(rows = vars(placement_method)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    scale_x_continuous(labels = scales::percent) +
    labs(
      x = "satisfied_ratio",
      y = "number functions provisioned"
    ) +
    geom_point(aes(size = n, color = run, fill = run)) +
    geom_line(aes(linetype = env), alpha = .1)
  # stat_summary(aes(color = placement_method, fill = placement_method), fun = mean, geom = "bar", alpha = 0.5)
  return(p)
}

output_number_requests_total <- function(respected_sla, node_levels) {
  df <- respected_sla %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(total = sum(total)) %>%
    inner_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(n = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    extract_context()

  p <- ggplot(data = df, aes(x = placement_method, y = total, alpha = 1)) +
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
    geom_point(aes(size = n, color = placement_method, fill = placement_method)) +
    geom_line(aes(group = run, linetype = env), alpha = .2) +
    stat_summary(aes(color = placement_method, fill = placement_method), fun = mean, geom = "bar", alpha = 0.5)

  return(p)
}

output_mean_time_to_deploy_simple_total <- function(deployment_times, node_levels) {
  df <- deployment_times %>%
    inner_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(n = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    extract_context()

  p <- ggplot(data = df, aes(alpha = 1)) +
    facet_grid(cols = vars(placement_method)) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Mean time to deploy (log10(ms))",
      x = "fog net size",
    ) +
    # scale_y_continuous(trans = "log10") +
    scale_y_log10(
      minor_breaks = rep(1:9, 4) * (10^rep(0:3, each = 9)),
      guide = "prism_minor"
    ) +
    guides(colour = guide_legend(ncol = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    geom_quasirandom(aes(shape = run, color = env, x = as.factor(n), y = value), method = "tukey", alpha = .2) +
    geom_boxplot(aes(x = as.factor(n), y = value, alpha = 0.8), outlier.shape = NA)

  return(p)
}
