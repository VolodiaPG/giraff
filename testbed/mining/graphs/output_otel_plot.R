output_otel_plot <- function(spans) {
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
    pivot_wider(names_from = field, values_from = value_raw) %>%
    mutate(
      duration = as.difftime(as.numeric(duration_nano) / 1e9, unit = "secs")
    ) %>%
    mutate(attributes = lapply(attributes, fromJSON)) %>%
    unnest_wider(attributes) %>%
    mutate(end_timestamp = timestamp + duration)

  # Log(df %>% select(span.name, retries) %>% sample_n(10))
  #
  # Log("There are duration NAs:")
  # nas <- df %>% filter(is.na(duration)) %>% count()
  # Log(nas$n)
  #
  # Log(colnames(df))

  # all_keys <- unique(unlist(lapply(toto$toto, names)))
  # Log(all_keys)

  # df_spans_raw <- df %>%
  #   filter(startsWith(span.name, "FLAME") & endsWith(span.name, "...")) %>%
  #   mutate(span.name = substring(span.name, 1, nchar(span.name) - 3)) %>%
  #   select(span.name, folder, duration, trace_id, retries_left)
  #
  # df_spans_raw2 <- df %>%
  #   filter(startsWith(span.name, "...FLAME")) %>%
  #   mutate(span.name = substring(span.name, 4)) %>%
  #   select(span.name, folder, end_duration = duration, trace_id, service.instance.id)
  #
  # df_spans <- df_spans_raw %>%
  #   inner_join(df_spans_raw2) %>%
  #   mutate(dead_time = duration - end_duration)
  #
  # # Log(df_spans %>% select(span.name, dead_time, duration, end_duration) %>% sample_n(5))
  #
  # df <- df_spans %>%
  #   mutate(duration = dead_time)

  # df <- df %>%
  #   group_by(folder, service.instance.id, span.name) %>%
  #   summarise(duration = mean(duration))

  # df_lines <- df %>%
  #   ungroup()
  #
  # df_lines <- df_lines %>%
  #   select(span_id, parent_span_id, folder, duration, span.name, service.instance.id, trace_id) %>%
  #   left_join(df_lines %>%
  #       select(parent_span.name = span.name, parent_duration = duration, folder, parent_span_id = span_id, parent_service.instance.id = service.instance.id))

  offs <- as.numeric(factor(df$trace_id))
  offscale <- (offs - mean(unique(offs))) * 0.005

  p <- ggplot(
    data = df,
    aes(alpha = 1, x = timestamp, y = service.namespace, color = span.name)
  ) +
    geom_segment(
      aes(xend = end_timestamp, yend = service.namespace),
      position = position_nudge(y = offscale),
      alpha = 0.3
    ) +
    # geom_point(position = position_nudge(y = offscale)) +
    geom_point(
      aes(x = end_timestamp),
      size = 0.3,
      position = position_nudge(y = offscale)
    ) +
    theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    labs(
      x = "Span",
      y = "Time",
    ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      )
    ) +
    theme(
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    scale_y_discrete() +
    scale_size(limits = c(2, 6)) +
    scale_x_continuous() +
    guides(colour = guide_legend(nrow = 1))

  p
}
