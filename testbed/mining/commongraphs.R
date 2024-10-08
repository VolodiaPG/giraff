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
      # x_centered = mean(x_centered),
      y_centered = mean(y),
      label = unique(placement_method)
    )

  jains <- df %>%
    ggplot(aes(alpha = 1, x = placement_method, y = y, color = env, fill = env)) +
    # facet_grid(rows = vars(env)) +
    # geom_hline(yintercept = max(earnings$worst_case), color = "black") +
    labs(
      x = "placement methods",
      y = "Jain's index"
    ) +
    scale_alpha_continuous(guide = "none") +
    guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    stat_ellipse(type = "norm", geom = "polygon", alpha = .1) +
    # geom_text(data = df_centroid, check_overlap = TRUE) +
    #  geom_point(aes(size = nodes, shape = env))
    geom_quasirandom(method = "tukey", alpha = .4)
  # geom_line(aes(group = run, linetype = run), alpha = .2)
  # stat_summary(aes(color = placement_method, fill = placement_method), fun = mean, geom = "bar", position = "dodge", alpha = 0.5)

  return(jains)
}

output_jains_anova_plot <- function(earnings, functions_all_total, node_levels) {
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
    ungroup() %>%
    select(placement_method, env, run, score, nodes) %>%
    rename(jains_index = score)

  create_metric_comparison_plot(
    data = df,
    metric_col = "placement_method",
    group_col = "env",
    value_col = "jains_index",
    node_col = "nodes",
    title = "Jain's Fairness Index by Placement Method",
    y_label = "Jain's Fairness Index",
    y_suffix = ""
  )
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
    mutate(chained = acceptable_chained) %>%
    select(metric_group_group, metric_group, folder, chained) %>%
    summarise(chained = sum(chained)) %>%
    inner_join(respected_sla %>%
      group_by(metric_group, metric_group_group, folder) %>%
      filter(prev_function == "<iot_emulation>") %>%
      select(metric_group_group, metric_group, folder, total) %>%
      summarise(total = sum(total))) %>%
    mutate(satisfied_ratio = chained / total) %>%
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


  p <- ggplot(data = df, aes(alpha = 1, x = x, y = y)) +
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
    # geom_text(data = df_centroid, aes(label = label, color = label), check_overlap = TRUE) +
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

output_mean_time_to_deploy_simple_total <- function(deployment_times, node_levels, paid_functions) {
  df <- deployment_times %>%
    inner_join(node_levels %>%
      group_by(metric_group, metric_group_group, folder) %>%
      summarise(n = n()), by = c("folder", "metric_group", "metric_group_group")) %>%
    left_join(paid_functions %>%
      group_by(metric_group, metric_group_group, folder) %>%
      rename(timestamp_paid = timestamp, paid = value), by = c("folder", "metric_group", "metric_group_group", "docker_fn_name", "sla_id")) %>%
    mutate(kube_overhaead = difftime(timestamp, timestamp_paid, units = "secs")) %>%
    mutate(placement_overhead = value / 1000 - kube_overhaead) %>%
    extract_context() %>%
    mutate(y = value) %>%
    ungroup() %>%
    mutate(y_centered = (y - mean(y, na.rm = TRUE)) / sd(y, na.rm = TRUE))

  p <- ggplot(data = df, aes(alpha = 1, x = as.factor(n), y = placement_overhead, color = placement_method)) +
    facet_grid(cols = vars(placement_method)) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      y = "Mean time to deploy (seconds)",
      x = "Fog network size (number of nodes)",
      title = "Single Function Deployment Time vs. Network Size",
      subtitle = "Comparison across different placement methods",
      caption = "Lower values indicate faster deployment",
      color = "Placement Method"
    ) +
    theme(
      legend.position = "right",
      legend.title = element_text(face = "bold"),
      legend.text = element_text(size = 8)
    ) +
    # coord_cartesian(ylim = c(-1, 1), clip = "on") +
    guides(colour = guide_legend(ncol = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    # geom_quasirandom(aes(shape = run, color = env, x = as.factor(n), y = value), method = "tukey", alpha = .2) +
    geom_boxplot(aes(alpha = 0.8), outlier.shape = NA) +
    geom_violin(aes(alpha = 0.8), outlier.shape = NA)
  # geom_text(data = df %>% filter(y_centered > 1), aes(y = 1, label = "+hidden"), nudge_x = 0.2)

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
    summarise(
      x_centered = mean(x_centered),
      y_centered = mean(y_centered),
      label = unique(placement_method)
    )

  p <- ggplot(data = df, aes(x = x_centered, y = y_centered, color = placement_method, fill = placement_method, alpha = 1)) +
    scale_alpha_continuous(guide = "none") +
    scale_size_continuous(guide = "none") +
    labs(
      x = "Total Requested Functions (Centered-reduced)",
      y = "Total Requests (Centered-reduced)",
      color = "Placement Method",
      fill = "Placement Method",
      size = "Number of Nodes"
    ) +
    theme(
      legend.position = "right",
      legend.box = "vertical",
      legend.margin = margin(5, 5, 5, 5),
      legend.box.margin = margin(0, 0, 0, 0),
      # legend.background = element_rect(fill = "white", color = "gray80"),
      legend.title = element_text(face = "bold"),
      legend.text = element_text(size = 8),
      axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    guides(
      color = guide_legend(order = 1),
      fill = guide_legend(order = 1),
      size = guide_legend(order = 2)
    ) +
    stat_ellipse(type = "norm", geom = "polygon", alpha = .1) +
    geom_text(data = df_centroid, aes(label = label, x = x_centered, y = y_centered), check_overlap = TRUE, show.legend = FALSE) +
    geom_point(aes(size = n))

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
      x_centered = mean(x),
      y_centered = mean(y),
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

output_placement_method_comparison <- function(respected_sla, functions_total, node_levels, bids_won_function, raw_deployment_times) {
  df <- respected_sla %>%
    group_by(chain_id, folder, metric_group, metric_group_group) %>%
    summarise(
      all_on_time = sum(acceptable_chained),
      avg_latency = mean(as.numeric(measured_latency)),
      total = sum(total),
      .groups = "drop"
    ) %>%
    mutate(
      respected_sla = all_on_time == total,
    ) %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(
      not_respected_slas = 1 - (sum(respected_sla) / n()),
      avg_latency = mean(avg_latency),
      .groups = "drop"
    ) %>%
    inner_join(
      functions_total %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(
          total_requested = sum(n),
          provisioned = sum(n[status == "provisioned"]),
          .groups = "drop"
        ) %>%
        mutate(functions_not_deployed = total_requested - provisioned),
      by = c("folder", "metric_group_group", "metric_group")
    ) %>%
    inner_join(
      node_levels %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(nodes = n(), .groups = "drop"),
      by = c("folder", "metric_group_group", "metric_group")
    ) %>%
    inner_join(
      bids_won_function %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(
          total_cost = sum(cost),
          total_functions = n_distinct(sla_id),
          .groups = "drop"
        ),
      by = c("folder", "metric_group_group", "metric_group")
    ) %>%
    inner_join(
      raw_deployment_times %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(avg_deployment_time = mean(value, na.rm = TRUE) / 1000, .groups = "drop"),
      by = c("folder", "metric_group_group", "metric_group")
    ) %>%
    mutate(total_cost = total_cost / total_functions)

  # Explicitly extract placement_method and env
  df <- df %>%
    extract_context() %>%
    select(placement_method, env, run, not_respected_slas, functions_not_deployed, nodes, total_cost, avg_latency, avg_deployment_time)

  # Center and reduce the variables
  df <- df %>%
    group_by(env, run) %>%
    group_by(env, run) %>%
    mutate(across(c(not_respected_slas, functions_not_deployed, total_cost, avg_latency, avg_deployment_time),
      list(scaled = ~ {
        ref_mean <- mean(.[placement_method == "auctionno_complication"], na.rm = TRUE)
        ref_sd <- sd(., na.rm = TRUE)
        (. - ref_mean) / ref_sd
      }),
      .names = "{col}_{fn}"
    )) %>%
    ungroup()


  # Define the order of metrics
  metric_order <- c("not_respected_slas", "functions_not_deployed", "total_cost", "avg_latency", "avg_deployment_time")

  # Reshape data for ggplot
  df_long <- df %>%
    correct_names() %>%
    select(placement_method, env, run, ends_with("_scaled"), not_respected_slas, functions_not_deployed, total_cost, avg_latency, avg_deployment_time, nodes) %>%
    pivot_longer(
      cols = ends_with("_scaled"),
      names_to = "metric",
      values_to = "value"
    ) %>%
    mutate(
      metric = sub("_scaled$", "", metric),
      metric = factor(metric, levels = metric_order),
      raw_value = case_when(
        metric == "not_respected_slas" ~ not_respected_slas,
        metric == "functions_not_deployed" ~ functions_not_deployed,
        metric == "total_cost" ~ total_cost,
        metric == "avg_latency" ~ avg_latency,
        metric == "avg_deployment_time" ~ avg_deployment_time
      )
    )

  # Calculate mean values and confidence intervals for each placement method
  df_mean <- df_long %>%
    group_by(placement_method, metric) %>%
    summarise(
      raw_value = mean(raw_value, na.rm = TRUE),
      se = sd(value, na.rm = TRUE) / sqrt(n()),
      value = mean(value, na.rm = TRUE),
      ci_lower = value - qt((1 - 0.05) / 2 + .5, df = n() - 1) * se,
      ci_upper = value + qt((1 - 0.05) / 2 + .5, df = n() - 1) * se,
      .groups = "drop"
    ) %>%
    mutate(x_jitter = as.numeric(factor(metric)) + seq(-0.1, 0.1, length.out = n()))

  # Create the parallel coordinates plot
  p <- ggplot(df_mean, aes(x = x_pos, y = value, color = placement_method, fill = placement_method, group = placement_method)) +
    scale_color_viridis_d() +
    scale_fill_viridis_d() +
    theme(
      panel.grid.major.x = element_blank(),
      panel.grid.minor = element_blank(),
      legend.position = "right",
      legend.box = "vertical",
      legend.direction = "vertical",
      legend.spacing.y = unit(0.2, "cm"),
      axis.title.y = element_blank(),
      axis.title.x = element_blank(),
    ) +
    scale_x_discrete(
      labels = c(
        "not_respected_slas" = "\\footnotesize{SLA Violations}",
        "functions_not_deployed" = "\\footnotesize{Undeployed Functions}",
        "total_cost" = "\\footnotesize{Total Cost}",
        "avg_latency" = "\\footnotesize{Avg. Latency}",
        "avg_deployment_time" = "\\footnotesize{Avg. Deployment Time}"
      ),
      guide = guide_axis(n.dodge = 2)
    ) +
    scale_y_continuous(labels = function(x) paste0(x, " SD")) +
    labs(
      title = "Mean Placement Method Comparison (Centered around GIRAFF and Reduced)",
      subtitle = "Focus on relative differences to GIRAFF by centering each experiment's distribution around GIRAFF and dividing by the standard deviation.",
      color = "Placement Method"
    ) +
    geom_ribbon(aes(x = metric, ymin = ci_lower, ymax = ci_upper, fill = placement_method), color = NA, alpha = 0.2) +
    geom_line(aes(x = metric, linetype = ifelse(placement_method == "\\footnotesize{GIRAFF}", "solid", "dashed")), alpha = 0.8) +
    geom_point(aes(x = metric, text = sprintf(
      "<br>Metric: %s<br>Placement Method: %s<br>Standardized Value: %.2f SD<br>Raw Value: %.2f",
      metric, placement_method,
      value, raw_value
    )), alpha = 0.6, stroke = 0, size = 3) +
    guides(
      color = guide_legend(title = "Placement Method", nrow = 2),
      fill = "none",
      linetype = "none"
    ) +
    scale_linetype_identity()

  return(p)
}

output_mean_deployment_times <- function(raw_deployment_times, node_levels, respected_sla) {
  df <- raw_deployment_times %>%
    # Join with respected_sla to get the chain information
    inner_join(
      respected_sla %>%
        select(folder, metric_group, metric_group_group, docker_fn_name, sla_id, chain_id),
      by = c("folder", "metric_group", "metric_group_group", "sla_id")
    ) %>%
    # Group by folder, metric_group, metric_group_group, and chain_id
    group_by(folder, metric_group, metric_group_group, chain_id) %>%
    summarise(
      deployment_time = sum(value) / 1000, # Convert to seconds and sum for the entire chain
      .groups = "drop"
    ) %>%
    inner_join(
      node_levels %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(nodes = n(), .groups = "drop"),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    mutate(deployment_time_per_function = deployment_time) %>%
    group_by(folder, metric_group, metric_group_group) %>%
    mutate(
      nodes2 = n() # Calculate the mean number of functions deployed per node
    ) %>%
    ungroup() %>%
    extract_context() %>%
    correct_names()

  create_metric_comparison_plot(
    data = df,
    metric_col = "placement_method",
    group_col = "folder",
    value_col = "deployment_time_per_function",
    node_col = "nodes",
    title = "Mean Deployment Time per Function in Chain by Placement Method",
    y_suffix = " s"
  )
}

output_mean_respected_slas <- function(respected_sla, node_levels) {
  df <- respected_sla %>%
    group_by(chain_id, folder, metric_group, metric_group_group) %>%
    summarise(
      all_on_time = sum(acceptable_chained),
      total = sum(total),
      .groups = "drop"
    ) %>%
    mutate(
      respected_sla = all_on_time == total,
    ) %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(
      respected_slas = sum(respected_sla) / n(),
      .groups = "drop"
    ) %>%
    inner_join(
      node_levels %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(nodes = n(), .groups = "drop"),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    mutate(respected_slas = respected_slas * 100) %>%
    extract_context() %>%
    correct_names()

  create_metric_comparison_plot(
    data = df,
    metric_col = "placement_method",
    group_col = "folder",
    value_col = "respected_slas",
    node_col = "nodes",
    title = "Mean Respected SLAs by Placement Method",
    y_suffix = "%"
  )
}

output_mean_spending <- function(bids_won_function, node_levels, respected_sla) {
  df <- bids_won_function %>%
    # Join with respected_sla to get the chain information
    inner_join(
      respected_sla %>%
        select(folder, metric_group, metric_group_group, docker_fn_name, sla_id, chain_id),
      by = c("folder", "metric_group", "metric_group_group", "sla_id")
    ) %>%
    # Group by folder, metric_group, metric_group_group, and chain_id
    group_by(folder, metric_group, metric_group_group, chain_id) %>%
    summarise(
      total_cost = sum(cost),
      chain_length = n_distinct(docker_fn_name),
      .groups = "drop"
    ) %>%
    inner_join(
      node_levels %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(nodes = n(), .groups = "drop"),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    mutate(spending_per_chain = total_cost) %>%
    extract_context() %>%
    correct_names()

  create_metric_comparison_plot(
    data = df,
    metric_col = "placement_method",
    group_col = "folder",
    value_col = "spending_per_chain",
    node_col = "nodes",
    title = "Mean Spending per Function Chain by Placement Method and Network Size",
    y_label = "Spending per Chain",
    y_suffix = " units"
  )
}

output_mean_placed_functions_per_node <- function(functions_total, node_levels) {
  df <- functions_total %>%
    filter(status == "provisioned") %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(
      placed_functions = sum(n),
      .groups = "drop"
    ) %>%
    inner_join(
      node_levels %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(nodes = n(), .groups = "drop"),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    mutate(placed_functions_per_node = placed_functions / nodes) %>%
    extract_context() %>%
    correct_names()

  create_metric_comparison_plot(
    data = df,
    metric_col = "placement_method",
    group_col = "folder",
    value_col = "placed_functions_per_node",
    node_col = "nodes",
    title = "Mean Placed Functions per Node by Placement Method",
    y_suffix = ""
  )
}

output_deployed_functions_ratio_anova_plot <- function(functions_total, node_levels) {
  df <- functions_total %>%
    group_by(folder, metric_group, metric_group_group) %>%
    summarise(
      deployed = sum(n[status == "provisioned"]),
      asked = sum(n),
      .groups = "drop"
    ) %>%
    mutate(ratio = deployed / asked) %>%
    inner_join(
      node_levels %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(nodes = n(), .groups = "drop"),
      by = c("folder", "metric_group", "metric_group_group")
    ) %>%
    extract_context() %>%
    correct_names()

  create_metric_comparison_plot(
    data = df,
    metric_col = "placement_method",
    group_col = "folder",
    value_col = "ratio",
    node_col = "nodes",
    title = "Ratio of Deployed Functions to Asked Functions by Placement Method",
    y_label = "Ratio of Deployed to Asked Functions",
    y_suffix = ""
  )
}
