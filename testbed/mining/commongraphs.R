output_provisioned_simple <- function(functions_total, node_levels) {
  df <- functions_total %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(functions = sum(n)) %>%
    inner_join(
      functions_total %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(total = sum(n)),
      by = c("folder", "metric_group_group", "metric_group")
    ) %>%
    inner_join(node_levels %>% group_by(metric_group, metric_group_group, folder) %>% summarise(nodes = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    mutate(ratio = functions / total) %>%
    extract_context()

  provisioned <- df %>%
    ggplot(aes(x = placement_method, y = functions)) +
    # geom_quasirandom(method = "tukey", alpha = .2) +$
    facet_grid(rows = vars(env)) +
    labs(
      x = "Function",
      y = "ration of functions provisioned over requested number"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    geom_point(aes(size = nodes, color = run, fill = run)) +
    geom_line(aes(group = run, color = run), alpha = .2)
  # stat_summary(aes(color = env, fill = env), fun = mean, geom = "bar", alpha = 0.5)

  return(provisioned)
}

output_provisioned_simple_total <- function(functions_total, node_levels) {
  df <- functions_total %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group_group, metric_group) %>%
    summarise(n = sum(n)) %>%
    inner_join(
      functions_total %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(total = sum(n)),
      by = c("folder", "metric_group_group", "metric_group")
    ) %>%
    inner_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(nodes = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    extract_context() %>%
    mutate(ratio = n / total)

  provisioned <- df %>%
    ggplot(aes(x = placement_method, y = ratio)) +
    # geom_quasirandom(method = "tukey", alpha = .2) +$
    labs(
      x = "placement method",
      y = "Ratio of provisioned functions / total functions asked to provision"
    ) +
    scale_y_continuous(labels = scales::percent) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    geom_point(aes(size = nodes, fill = env, color = env)) +
    geom_line(aes(group = run, fill = env, color = env), alpha = .2) +
    geom_boxplot()
  # stat_summary(aes(color = metric_group, fill = metric_group, ), fun = mean, geom = "bar", alpha = 0.5)

  return(provisioned)
}

output_jains_simple <- function(earnings, functions_all_total, node_levels) {
  df <- earnings %>%
    left_join(
      functions_all_total %>%
        rename(total_requested_functions = total, ratio_requested_functions = ratio),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    left_join(
      node_levels %>%
        group_by(metric_group, metric_group_group, folder) %>%
        summarise(nodes = n()),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    extract_context() %>%
    mutate(x = total_requested_functions) %>%
    mutate(y = score) %>%
    group_by(env, run) %>%
    mutate(x_centered = (x - mean(x, na.rm = TRUE)) / sd(x, na.rm = TRUE)) %>%
    mutate(y_centered = (y - mean(y, na.rm = TRUE)) / sd(y, na.rm = TRUE))

  # Calculate the centroid of each ellipse
  df_centroid <- df %>%
    group_by(placement_method) %>%
    # filter(n() > 3) %>% # Ensure at least 3 points in each group
    summarise(
      x_centered = mean(x_centered),
      y_centered = mean(y_centered),
      label = unique(placement_method)
    )

  jains <- df %>%
    ggplot(aes(alpha = 1, x = x_centered, y = y_centered, color = placement_method, fill = placement_method)) +
    # facet_grid(rows = vars(env)) +
    # geom_hline(yintercept = max(earnings$worst_case), color = "black") +
    labs(
      x = "center reduced placed functions for each pair of (env, run)",
      y = "Jain's index"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    stat_ellipse(type = "norm", geom = "polygon", alpha = .1) +
    geom_text(data = df_centroid, aes(label = label, color = label), check_overlap = TRUE) +
    geom_point(aes(size = nodes, shape = env))
  # geom_quasirandom(aes(shape = env, color = placement_method), method = "tukey", alpha = .4) +
  # geom_line(aes(group = run, linetype = run), alpha = .2)
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
      y = "Function cost (log10)",
      x = "Placement method",
    ) +
    scale_y_log10(
      minor_breaks = rep(1:9, 4) * (10^rep(0:3, each = 9)),
      guide = "prism_minor"
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
    inner_join(node_levels %>% group_by(metric_group, metric_group_group, folder) %>% summarise(nodes = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    extract_context()

  p <- ggplot(data = df, aes(x = placement_method, y = total, alpha = 1)) +
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
    geom_point(aes(color = placement_method, fill = placement_method, )) +
    geom_line(aes(group = run), alpha = .2) +
    stat_summary(aes(color = placement_method, fill = placement_method, ), fun = mean, geom = "bar", alpha = 0.5)

  return(p)
}

output_respected_data_plot_total <- function(respected_sla, functions_all_total, node_levels) {
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
    left_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(nodes = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    left_join(
      functions_all_total %>%
        rename(total_requested_functions = total, ratio_requested_functions = ratio),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    ungroup() %>%
    mutate(x = total_requested_functions) %>%
    mutate(y = satisfied_ratio) %>%
    group_by(env, run) %>%
    mutate(x_centered = (x - mean(x, na.rm = TRUE)) / sd(x, na.rm = TRUE)) %>%
    mutate(y_centered = (y - mean(y, na.rm = TRUE)) / sd(y, na.rm = TRUE))

  # Calculate the centroid of each ellipse
  df_centroid <- df %>%
    group_by(placement_method) %>%
    summarise(
      x_centered = mean(x_centered),
      y_centered = mean(y_centered),
      label = unique(placement_method)
    )


  p <- ggplot(data = df, aes(alpha = 1, x = x_centered, y = y_centered)) +
    # facet_grid(rows = vars(env)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    # scale_y_continuous(labels = scales::percent) +
    labs(
      y = "SLA satisfaction ratio (centered reduced)",
      x = "ratio of functions requested to place (centered reduced)"
    ) +
    # geom_line(aes(group = env, linetype = env), alpha = .1) +
    stat_ellipse(aes(color = placement_method, fill = placement_method), type = "norm", geom = "polygon", alpha = .1) +
    geom_text(data = df_centroid, aes(label = label, color = label), check_overlap = TRUE) +
    geom_point(aes(shape = env, size = nodes, color = placement_method, fill = placement_method))

  # stat_summary(fun = mean, geom = "bar", alpha = 0.25)
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
    # facet_grid(rows = vars(n)) +
    scale_alpha_continuous(guide = "none") +
    scale_size_continuous(guide = "none") +
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
    # guides(colour = guide_legend(nrow = 1)) +
    geom_point(aes(size = n, color = run, fill = run)) +
    geom_line(aes(group = interaction(run, env), color = run), alpha = .2)
  # stat_summary(aes(color = placement_method, fill = placement_method), fun = mean, geom = "bar", alpha = 0.5)

  return(p)
}

output_mean_time_to_deploy_simple_total <- function(deployment_times, node_levels) {
  df <- deployment_times %>%
    inner_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(n = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    extract_context() %>%
    mutate(y = value) %>%
    # group_by(env, run) %>%
    ungroup() %>%
    mutate(y_centered = (y - mean(y, na.rm = TRUE)) / sd(y, na.rm = TRUE))


  p <- ggplot(data = df, aes(alpha = 1)) +
    facet_grid(cols = vars(placement_method), x = as.factor(n), y = y_centered) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Mean time to deploy",
      x = "fog net size",
    ) +
    coord_cartesian(ylim = c(-1, 1), clip = "on") +
    guides(colour = guide_legend(ncol = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    # geom_quasirandom(aes(shape = run, color = env, x = as.factor(n), y = value), method = "tukey", alpha = .2) +
    geom_boxplot(aes(alpha = 0.8), outlier.shape = NA) +
    geom_violin(aes(alpha = 0.8), outlier.shape = NA) +
    geom_text(data = df %>% filter(y_centered > 1), aes(x = as.factor(n), y = 1, label = "+hidden"), nudge_x = 0.2)

  return(p)
}

output_requests_served_v_provisioned <- function(respected_sla, functions_total, node_levels) {
  df <- respected_sla %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(all_requests = sum(total)) %>%
    inner_join(
      functions_total %>%
        filter(status == "provisioned") %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(functions = sum(n)),
      by = c("folder", "metric_group_group", "metric_group")
    ) %>%
    inner_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(n = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    extract_context() %>%
    mutate(x = functions) %>%
    mutate(y = all_requests) %>%
    group_by(env, run) %>%
    mutate(x_centered = (x - mean(x, na.rm = TRUE)) / sd(x, na.rm = TRUE)) %>%
    mutate(y_centered = (y - mean(y, na.rm = TRUE)) / sd(y, na.rm = TRUE))

  df_centroid <- df %>%
    group_by(placement_method) %>%
    # filter(n() > 3) %>% # Ensure at least 3 points in each group
    summarise(
      x_centered = mean(x_centered),
      y_centered = mean(y_centered),
      label = unique(placement_method)
    )

  p <- ggplot(data = df, aes(x = x_centered, y = y_centered, color = placement_method, fill = placement_method, alpha = 1)) +
    scale_alpha_continuous(guide = "none") +
    scale_size_continuous(guide = "none") +
    labs(
      x = "total requested functions",
      y = "total requests",
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
    # guides(colour = guide_legend(nrow = 1)) +
    stat_ellipse(type = "norm", geom = "polygon", alpha = .1) +
    geom_text(data = df_centroid, aes(label = label, color = label), check_overlap = TRUE) +
    geom_point(aes(size = n))
  # stat_summary(aes(color = placement_method, fill = placement_method), fun = mean, geom = "bar", alpha = 0.5)

  return(p)
}

output_non_respected <- function(respected_sla, functions_all_total, node_levels) {
  df <- respected_sla %>%
    filter(acceptable == 0 & service_oked > 0) %>%
    group_by(metric_group, metric_group_group, folder) %>%
    select(metric_group_group, metric_group, folder, acceptable_chained) %>%
    summarise(misplaced = n()) %>%
    left_join(
      respected_sla %>%
        group_by(metric_group_group, metric_group, folder) %>%
        # filter(acceptable != total) %>%
        filter(total - acceptable - all_errors != 0) %>%
        summarise(at_least_one_rejection = n()),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    extract_context() %>%
    left_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(nodes = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    left_join(
      functions_all_total %>%
        rename(total_requested_functions = total, ratio_requested_functions = ratio),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    # mutate(x = total_requested_functions) %>%
    mutate(x = at_least_one_rejection) %>%
    mutate(y = misplaced) %>%
    group_by(env, run) %>%
    mutate(x_centered = (x - mean(x, na.rm = TRUE)) / sd(x, na.rm = TRUE)) %>%
    mutate(y_centered = (y - mean(y, na.rm = TRUE)) / sd(y, na.rm = TRUE))

  df_centroid <- df %>%
    group_by(placement_method) %>%
    # filter(n() > 3) %>% # Ensure at least 3 points in each group
    summarise(
      x_centered = mean(x_centered),
      y_centered = mean(y_centered),
      label = unique(placement_method)
    )

  p <- ggplot(data = df, aes(alpha = 1, x = x_centered, y = y_centered, color = placement_method, fill = placement_method)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    labs(
      x = "# of functions with at least 1 non-respected SLA (center reduced)",
      y = "# of functions that miss their target for ALL of their requests (center_reduced)"
    ) +
    stat_ellipse(type = "norm", geom = "polygon", alpha = .1) +
    geom_text(data = df_centroid, aes(label = label, color = label), check_overlap = TRUE) +
    geom_point(aes(size = nodes))
  # geom_line(aes(group = run, linetype = run), alpha = .1)
  # stat_summary(fun = mean, geom = "bar", alpha = 0.25)
  return(p)
}
