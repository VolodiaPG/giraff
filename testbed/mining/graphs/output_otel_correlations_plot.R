output_otel_correlations_plot <- function(spans, latency) {
  df_spans_raw <- spans %>%
    filter(startsWith(span.name, "FLAME") & endsWith(span.name, "...")) %>%
    mutate(span.name = substring(span.name, 1, nchar(span.name) - 3)) %>%
    select(
      span.name,
      folder,
      out_duration = duration,
      trace_id,
      service.namespace,
      timestamp
    )

  df_spans_raw2 <- spans %>%
    filter(startsWith(span.name, "...FLAME")) %>%
    mutate(span.name = substring(span.name, 4)) %>%
    select(
      span.name,
      folder,
      in_duration = duration,
      trace_id,
      service.namespace
    )

  df_spans <- df_spans_raw %>%
    inner_join(df_spans_raw2) %>%
    mutate(duration = out_duration - in_duration) %>%
    select(
      trace_id,
      timestamp,
      span.name,
      folder,
      duration,
      service.namespace
    ) %>%
    group_by(span.name, service.namespace, folder) %>%
    summarise(
      mean_duration = mean(duration),
      median_duration = median(duration),
      .groups = "drop"
    ) %>%
    left_join(latency)

  # df <- df_spans_raw %>%
  #   # filter(span.name %in% c("create_machine", "start_processing_requests")) %>%
  #   mutate(duration = end_timestamp - timestamp) %>%
  #   left_join(df_spans) %>%
  #   left_join(latency)
  #
  Log(df_spans %>% select(span.name, mean_duration, latency))

  # Create the correlation plot between for each of the unique span.names with the latency, the duration. Use ggplot and a heatmap
  # Prepare data for correlation analysis
  # df <- df %>%
  #   select(span.name, duration) %>%
  #   filter(!is.na(span.name) & !is.na(duration)) %>%
  #   mutate(duration_numeric = as.numeric(duration))

  # Group by span name to get statistics
  correlation_data <- df %>%
    filter(
      !span.name %in% c("create_machine", "start_processing_requests")
    ) %>%
    group_by(service.namespace, span.name) %>%
    summarise(
      mean_duration = mean(duration, na.rm = TRUE),
      mean_latency = mean(latency, na.rm = TRUE),
      median_duration = median(duration, na.rm = TRUE),
      # n_requests = n(),
      .groups = "drop"
    ) %>%
    pivot_longer(
      cols = c(mean_duration, median_duration),
      names_to = "name",
      values_to = "value"
    )

  Log(correlation_data)

  correlation_data_create_machine <- df %>%
    group_by(service.namespace) %>%
    filter(span.name == "create_machine") %>%
    summarise(
      n_create_machine = n(),
      .groups = "drop"
    )

  correlation_data_processing_requests <- df %>%
    group_by(service.namespace) %>%
    filter(span.name == "start_processing_requests") %>%
    summarise(
      n_prosessing_requests = n(),
      .groups = "drop"
    )

  heatmap_data <- correlation_data %>%
    left_join(correlation_data_create_machine) %>%
    left_join(correlation_data_processing_requests)

  Log(heatmap_data)

  heatmap_data <- summary_data %>%
    select(span.name, mean_duration, median_duration) %>%
    pivot_longer(
      cols = c(mean_duration, median_duration),
      names_to = "metric",
      values_to = "value"
    ) %>%
    mutate(
      metric = case_when(
        metric == "mean_duration" ~ "Mean Duration",
        metric == "median_duration" ~ "Median Duration"
      )
    )

  # Create heatmap
  correlation_plot <- ggplot(
    heatmap_data,
    aes(x = metric, y = reorder(span.name, value), fill = value)
  ) +
    geom_tile(color = "white", size = 0.2) +
    # geom_text() +
    scale_fill_viridis_c(name = "Duration (s)", option = "plasma") +
    labs(
      title = "Span Duration Metrics Comparison",
      x = "Metric",
      y = "Span Name"
    ) +
    theme(
      axis.text.x = element_text(angle = 45, hjust = 1),
      panel.grid = element_blank(),
      plot.title = element_text(hjust = 0.5)
    )

  return(correlation_plot)
}
