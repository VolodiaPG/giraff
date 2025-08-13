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
  # Log(
  #   df %>%
  #     ungroup() %>%
  #     select(span.name) %>%
  #     distinct() %>%
  #     sample_n(10)
  # )

  df_spans_raw <- df %>%
    filter(startsWith(span.name, "FLAME") & endsWith(span.name, "...")) %>%
    mutate(span.name = substring(span.name, 1, nchar(span.name) - 3)) %>%
    select(
      span.name,
      folder,
      start_timestamp = timestamp,
      start_end_timestamp = end_timestamp,
      trace_id,
      service.namespace
    )

  df_spans_raw2 <- df %>%
    filter(startsWith(span.name, "...FLAME")) %>%
    mutate(span.name = substring(span.name, 4)) %>%
    select(
      span.name,
      folder,
      end_timestamp = timestamp,
      end_end_timestamp = timestamp,
      trace_id,
      service.namespace
    )

  df_spans <- df_spans_raw %>%
    inner_join(df_spans_raw2) %>%
    mutate(duration = end_timestamp - start_timestamp) %>%
    mutate(duration = start_end_timestamp - end_end_timestamp + duration) %>%
    select(
      span.name,
      folder,
      duration,
      service.namespace
    )

  df <- df %>%
    filter(span.name %in% c("create_machine")) %>%
    mutate(duration = end_timestamp - timestamp) %>%
    # filter(span.name %in% c("create_machine", "start_processing_requests")) %>%
    full_join(df_spans)

  # Log(df)

  offs <- as.numeric(factor(df$trace_id))
  offscale <- (offs - mean(unique(offs))) * 0.005

  p <- ggplot(
    data = df,
    aes(
      color = span.name,
      # fill = span.name,
      # x = service.namespace,
      x = duration,
      group = span.name
    )
  ) +
    # geom_col(na.rm = TRUE) +
    # geom_boxplot(position = position_dodge2()) +
    # geom_freqpoly() +
    # geom_density(alpha = 0.5) +
    stat_ecdf(geom = "step", pad = TRUE) +
    # geom_segment(
    #   data = df_create_machine,
    #   aes(
    #     # x = timestamp,
    #     xend = end_timestamp,
    #     # y = service.namespace,
    #     yend = service.namespace
    #   ),
    #   position = position_nudge(y = offscale),
    #   alpha = 1,
    #   color = "black"
    # ) +
    # geom_segment(
    #   data = df_processing,
    #   aes(
    #     # x = timestamp,
    #     xend = end_timestamp,
    #     # y = service.namespace,
    #     yend = service.namespace
    #   ),
    #   position = position_nudge(y = offscale),
    #   alpha = 0.1,
    #   color = "black"
    # ) +
    # geom_segment(
    #   data = df_spans,
    #   aes(
    #     x = end_end_timestamp,
    #     xend = start_end_timestamp,
    #     y = service.namespace,
    #     yend = service.namespace
    #   ),
    #   position = position_nudge(y = offscale),
    #   alpha = 0.4
    # ) +
    # geom_point(position = position_nudge(y = offscale)) +
    # geom_point(
    #   aes(x = end_timestamp),
    #   # size = 0.3,
    #   position = position_nudge(y = offscale)
    # ) +
    # theme(legend.position = "none") +
    scale_alpha_continuous(guide = "none") +
    # labs(
    #   x = "Span",
    #   y = "Time",
    # ) +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        # size = 0.2,
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
    guides(colour = guide_legend(nrow = 1))

  p
}
