output_otel_budget_plot <- function(spans) {
  df <- spans %>%
    ungroup() %>%
    select(
      folder,
      timestamp,
      service.name,
      field,
      value_raw,
      span_id,
      trace_id
    ) %>%
    group_by(folder) %>%
    pivot_wider(names_from = field, values_from = value_raw) %>%
    mutate(
      duration = as.difftime(as.numeric(duration_nano) / 1e9, unit = "secs")
    ) %>%
    mutate(attributes = lapply(attributes, fromJSON)) %>%
    unnest_wider(attributes) %>%
    select(folder, timestamp, new_budget, service.namespace) %>%
    filter(!is.na(new_budget))

  # Log(df %>% sample_n(5))
  #
  p <- ggplot(
    data = df,
    aes(
      x = timestamp,
      y = new_budget,
      color = service.namespace,
    )
  ) +
    geom_line() +
    geom_point() +
    # theme(legend.position = "none") +
    # scale_alpha_continuous(guide = "none") +
    labs(
      title = paste(
        "Budget evolution of each application\n",
        unique(df$folder)
      ),
      x = "Time",
      y = "Budget",
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
    )
  # scale_color_viridis(discrete = TRUE) +
  # scale_fill_viridis(discrete = TRUE) +
  # scale_x_discrete() +
  # scale_x_continuous() +
  # scale_size(limits = c(2, 6)) +
  # cale_y_continuous() +
  # guides(colour = guide_legend(nrow = 1))

  p
}
