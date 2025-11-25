big_output_nb_success_vs_nb_functions_plot <- function(
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

  outliers <- df %>%
    group_by(env, env_live) %>%
    filter(
      requests > quantile(requests, 0.99) |
        requests < quantile(requests, 0.01) |
        nb_functions > quantile(nb_functions, 0.99) |
        nb_functions < quantile(nb_functions, 0.01)
    )

  ggplot(df, aes(x = success_rate, y = nb_functions)) +
    stat_density_2d(
      aes(
        fill = after_stat(density)
      ),
      bounds = c(0, Inf),
      geom = "raster",
      contour = FALSE,
      interpolate = TRUE
    ) +
    stat_density_2d(color = "white", alpha = 0.5, bins = 5) +
    geom_point(
      data = outliers,
      color = "red",
      size = 0.2,
      alpha = 0.7,
      shape = "cross"
    ) +
    scale_fill_viridis(option = "turbo") +
    # scale_x_log10(
    #   breaks = trans_breaks("log10", function(x) 10^x),
    #   labels = log10_labels()
    # ) +
    scale_y_log10() +
    scale_x_continuous(labels = scales::percent, n.breaks = 3) +
    guides(y = guide_axis_logticks(negative.small = 1)) +
    facet_grid(cols = vars(env_live), rows = vars(env)) +
    labs(
      fill = "Density",
      # x = "End-user requests to the gateway",
      x = "Ratio of successful requests to total requests per application",
      y = "Nb functions per application"
    )
}
