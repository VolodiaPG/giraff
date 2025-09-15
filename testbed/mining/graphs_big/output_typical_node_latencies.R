big_output_typical_node_latencies_plot <- function(latency, node_levels) {
  Log(colnames(latency))
  Log(colnames(node_levels))

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
    group_by(folder, metric_group, source_level, destination_level) %>%
    summarise(latency = mean(latency)) %>%
    extract_context() %>%
    filter(source_level < destination_level) %>%
    mutate(latency = latency / 1000) %>%
    group_by(run, source_level, destination_level) %>%
    summarise(latency = mean(latency))

  ggplot(
    data = df,
    aes(
      x = interaction(
        source_level,
        destination_level,
        sep = " $\\rightarrow$ "
      ),
      y = latency,
      color = run
    ),
  ) +
    # facet_grid(rows = vars(env), cols = vars(env_live)) +
    # geom_beeswarm() +
    geom_quasirandom(method = "tukey") +
    theme(
      legend.background = element_rect(
        fill = alpha("white", .7),
        size = 0.2,
        color = alpha("white", .7)
      ),
      legend.position = "none",
      legend.spacing.y = unit(0, "cm"),
      legend.margin = margin(0, 0, 0, 0),
      legend.box.margin = margin(-10, -10, -10, -10)
      # axis.text.x = element_text(angle k 0, vjust = 1, hjust = 1)
    ) +
    labs(
      title = paste("Typical Latencies between nodes (symmetric)"),
      x = "Node level source $\\rightarrow$ destination",
      y = "Latency (s)"
    )
}
