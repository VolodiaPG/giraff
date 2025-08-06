output_otel_budget_plot <- function(spans) {
  df <- spans %>%
    ungroup() %>%
    select(folder, timestamp, service.name, field, value_raw, span_id, trace_id) %>%
    pivot_wider(names_from = field, values_from = value_raw) %>%
    mutate(duration = as.difftime(as.numeric(duration_nano) / 1e9, unit = "secs")) %>%
    mutate(attributes = lapply(attributes, fromJSON)) %>%
    unnest_wider(attributes)

  Log(df %>% sample_n(5))

  Log(colnames(df))

  p <- ggplot(data = df, aes(alpha = 1, x = timestamp, y = as.numeric(budget), color = service.namespace, group = service.namespace)) +
    # facet_grid(~var_facet) +
    # geom_beeswarm(aes(size = retries)) +
    # geom_beeswarm() +
    geom_point() +
    # geom_segment(data = df_lines, aes(x = parent_span.name, y = parent_duration, xend = span.name, yend = duration, color = trace_id), alpha = 0.2) +
    # geom_quasirandom(method = "tukey", alpha = .8) +
    geom_line(alpha = 0.2) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      x = "Span",
      y = "Time",
    ) +
    theme(legend.background = element_rect(
      fill = alpha("white", .7),
      size = 0.2, color = alpha("white", .7)
    )) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    # scale_x_discrete() +
    scale_x_continuous() +
    scale_size(limits = c(2, 6)) +
    scale_y_continuous() +
    guides(colour = guide_legend(nrow = 1))

  p
}