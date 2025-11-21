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

  outliers <- df %>%
    group_by(env, env_live) %>%
    filter(
      requests > quantile(requests, 0.99) |
        requests < quantile(requests, 0.01) |
        nb_functions > quantile(nb_functions, 0.99) |
        nb_functions < quantile(nb_functions, 0.01)
    )

  ggplot(df, aes(x = requests, y = nb_functions)) +
    stat_density_2d(
      aes(
        fill = after_stat(density)
      ),
      geom = "raster",
      contour = FALSE,
      interpolate = TRUE
    ) +
    # stat_density_2d(color = "white", alpha = 0.5, bins = 5) +
    geom_point(
      data = outliers,
      color = "red",
      size = 0.2,
      alpha = 0.7,
      shape = "cross"
    ) +
    scale_fill_viridis_c(option = "turbo") +
    scale_x_log10(
      breaks = trans_breaks("log10", function(x) 10^x),
      labels = log10_labels()
    ) +
    scale_y_log10(
      breaks = trans_breaks("log10", function(x) 10^x),
      labels = log10_labels()
    ) +
    annotation_logticks() +
    facet_grid(cols = vars(env_live), rows = vars(env)) +
    labs(fill = "Density")
}
