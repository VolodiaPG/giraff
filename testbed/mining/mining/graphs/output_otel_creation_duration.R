output_otel_creation_duration <- function(spans) {
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
    filter(span.name %in% c("start_processing_requests")) %>%
    mutate(duration_numeric = as.numeric(duration)) %>%
    mutate(
      error = if ("otel.status_code" %in% names(df)) {
        ifelse(
          is.na(otel.status_code),
          FALSE,
          otel.status_code == "Error"
        )
      } else {
        FALSE # Default value if column does not exist
      }
    )

  Log("errors")
  Log(
    df %>%
      filter(error == TRUE) %>%
      select(folder, timestamp, service.namespace)
  )
  # mutate(duration = end_timestamp - timestamp) %>%
  # filter(span.name %in% c("create_machine", "start_processing_requests")) %>%
  # full_join(df_spans)

  # Log(
  #   df %>%
  #     select(service.namespace, duration, otel.status_code) %>%
  #     group_by(service.namespace)
  #   summarise(errors = sum(otel.status_code == "Error"), .groups = "drop")
  # )

  hull_data <- df %>%
    group_by(service.namespace) %>%
    slice(chull(timestamp, duration_numeric))

  ggplot(
    data = df,
    aes(
      x = timestamp,
      y = duration_numeric,
    )
  ) +
    geom_point(aes(color = error)) +
    # geom_segment(aes(
    #   x = timestamp,
    #   xend = end_timestamp,
    #   y = duration_numeric,
    #   yend = duration_numeric,
    #   color = error
    # )) +
    geom_polygon(
      data = hull_data,
      aes(
        x = timestamp,
        y = duration_numeric,
        fill = service.namespace
      ),
      color = NA,
      alpha = 0.2,
      inherit.aes = FALSE
    ) +
    scale_alpha_continuous(guide = "none") +
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
    # scale_color_viridis(discrete = TRUE) +
    labs(title = unique(df$folder)) +
    scale_fill_viridis(discrete = TRUE) +
    guides(colour = guide_legend(nrow = 1))
}
