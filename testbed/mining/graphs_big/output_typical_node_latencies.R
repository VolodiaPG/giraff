big_output_typical_node_latencies_plot <- function(latency, node_levels) {
  # Log(colnames(latency))
  # Log(colnames(node_levels))

  df <- latency %>%
    filter(field == "average") %>%
    adjust_timestamps() %>%
    rename(
      source = instance,
      destination = destination_name,
      latency = value
    ) %>%
    select(timestamp, source, destination, folder, metric_group, latency) %>%
    mutate(
      sorted_interaction = pmap_chr(
        list(source, destination),
        ~ paste(sort(c(...)), collapse = "_")
      )
    ) %>%
    group_by(folder, metric_group, source, destination) %>%
    summarise(latency = mean(latency)) %>%
    left_join(
      node_levels %>% rename(source_level = level_value),
      by = c("source" = "name", "folder")
    ) %>%
    left_join(
      node_levels %>% rename(destination_level = level_value),
      by = c("destination" = "name", "folder")
    ) %>%
    filter(source_level < destination_level) %>%
    mutate(latency = latency / 1000) %>%
    mutate(
      x = interaction(
        source_level,
        destination_level,
        sep = " $\\rightarrow$ "
      )
    ) %>%
    extract_context()

  df_max <- df %>%
    group_by(x) %>%
    summarise(max_latency = max(latency), min_latency = min(latency))

  df <- df %>%
    group_by(folder, metric_group, x, run) %>%
    summarise(latency = mean(latency)) %>%
    group_by(run, x) %>%
    summarise(latency = mean(latency))

  df_mean <- df %>%
    group_by(x) %>%
    summarise(latency = mean(latency))

  ggplot(
    data = df,
    aes(
      x = x,
    ),
  ) +
    geom_col(
      data = df_mean,
      aes(x = x, y = latency),
      position = position_dodge(width = 0.9),
      alpha = 0.8,
    ) +
    geom_point(
      aes(y = latency),
      position = position_dodge(width = 0.9),
    ) +
    geom_errorbar(
      data = df_max,
      aes(x = x, ymin = min_latency, ymax = max_latency),
      position = position_dodge(width = 0.9),
      width = 0.2
    ) +

    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.position = "none",
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10),
      axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)
    ) +
    labs(
      title = paste("Typical Latencies between nodes (symmetric)"),
      x = "Node level source $\\rightarrow$ destination",
      y = "Latency (s)"
    ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE)
}
