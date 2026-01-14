big_output_nb_success_vs_requests_plot <- function(
  nb_functions,
  nb_requests,
  nb_nodes
) {
  df <- nb_requests %>%
    extract_context() %>%
    group_by(folder, env, env_live, service.namespace) %>%
    summarise(requests = sum(requests), success = sum(success)) %>%
    mutate(success_rate = success / requests) %>%
    left_join(nb_nodes, by = c("folder")) %>%
    left_join(
      nb_functions %>% select(folder, service.namespace, nb_functions)
    ) %>%
    extract_env_name() %>%
    env_live_extract()

  original_levels <- levels(diamonds$env_live)
  wrapped_levels <- str_wrap(original_levels, width = 18)

  df <- df %>%
    mutate(
      wrapped_label = str_wrap(env_live, width = 18),
      env_live = fct_relevel(wrapped_label, wrapped_levels)
    )

  # Create bins manually per facet
  # bin_data <- df %>%
  #   group_by(env, env_live) %>%
  #   filter(
  #     !is.na(requests) &
  #       !is.na(success) &
  #       requests > 0 &
  #       success > 0
  #   ) %>%
  #   mutate(requests = log10(requests)) %>%
  #   mutate(success = log10(success)) %>%
  #   mutate(
  #     x_bin = cut(requests, breaks = 15),
  #     y_bin = cut(success, breaks = 15)
  #   ) %>%
  #   group_by(env, env_live, x_bin, y_bin) %>%
  #   summarise(
  #     x = mean(requests),
  #     y = mean(success),
  #     count = n(),
  #     .groups = "drop"
  #   ) # Perform Linear Interpolation per facet
  # interp_df <- bin_data %>%
  #   group_by(env, env_live) %>%
  #   group_map(
  #     function(data, keys) {
  #       surface <- akima::interp(
  #         x = data$x,
  #         y = data$y,
  #         z = data$count,
  #         linear = TRUE,
  #         extrap = FALSE,
  #         xo = seq(min(data$x), max(data$x), length.out = 200),
  #         yo = seq(min(data$y), max(data$y), length.out = 200)
  #       )
  #       data.frame(
  #         x = rep(surface$x, length(surface$y)),
  #         y = rep(surface$y, each = length(surface$x)),
  #         z = as.vector(surface$z),
  #         env = keys$env,
  #         env_live = keys$env_live
  #       )
  #     },
  #     .keep = TRUE
  #   ) %>%
  #   bind_rows() %>%
  #   filter(!is.na(x) & !is.na(y) & !is.na(z)) %>%
  #   mutate(x = (11^x)) %>%
  #   mutate(y = (10^y))

  # outliers <- df %>%
  #   group_by(env, env_live) %>%
  #   filter(
  #     requests > quantile(requests, 0.99) |
  #       requests < quantile(requests, 0.01) |
  #       nb_functions > quantile(nb_functions, 0.99) |
  #       nb_functions < quantile(nb_functions, 0.01)
  #   )
  ggplot(df, aes(x = requests, y = success)) +
    geom_hex(
      bins = 10,
      bounds = c(0, Inf),
      geom = "raster"
    ) +
    # ggplot(interp_df, aes(x = x, y = y, fill = z)) +
    #   geom_raster(na.rm = TRUE) +
    # geom_hex(stat = "identity") +

    scale_fill_viridis(option = "turbo") +
    scale_x_log10(
      breaks = trans_breaks("log10", function(x) 10^x),
      labels = log10_labels()
    ) +
    scale_y_log10(
      breaks = trans_breaks("log10", function(x) 10^x),
      labels = log10_labels()
    ) +
    guides(x = guide_axis_logticks(negative.small = 1)) +
    guides(y = guide_axis_logticks(negative.small = 1)) +
    facet_grid(cols = vars(env_live), rows = vars(env)) +
    labs(
      fill = "Nb",
      x = "End-user requests to the application",
      y = "Successful responses by the application"
    )
}
