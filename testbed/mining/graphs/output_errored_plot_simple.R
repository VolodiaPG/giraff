output_errored_plot_simple <- function(respected_sla, bids_won_function, node_levels) {
  df <- respected_sla %>%
    left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
    left_join(node_levels %>% rename(winner = name)) %>%
    mutate(y = server_errored / total) %>%
    mutate(pipeline = pipeline) %>%
    {
      .
    }

  p <- ggplot(data = df, aes(x = factor(pipeline), y = y, color = docker_fn_name, fill = docker_fn_name, alpha = 1)) +
    # facet_grid(rows = vars(pipeline)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    # scale_y_continuous(l:abels = scales::percent) +
    theme(axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)) +
    labs(
      x = "Placement method",
      y = "Mean satisfaction rate"
    ) +
    geom_violin()

  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
  }
  return(p)
}