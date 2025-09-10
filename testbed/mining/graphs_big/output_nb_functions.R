big_ouput_nb_functions_plot <- function(spans, degrades, errors) {
  Log(colnames(spans))

  df <- spans %>%
    extract_context() %>%
    select(
      env,
      env_live,
      folder,
      metric_group,
      service.namespace,
      service.instance.id
    ) %>%
    group_by(folder, metric_group, service.namespace, env, env_live) %>%
    distinct() %>%
    summarise(nb_functions = n())

  ggplot(
    data = df,
    aes(
      x = env_live,
      y = nb_functions,
      color = folder
    ),
  ) +
    facet_grid(rows = vars(env)) +
    geom_beeswarm() +
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
    )
}
