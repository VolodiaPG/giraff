output_respected_data_plot_simple <- function(
  respected_sla,
  bids_won_function,
  node_levels
) {
  df <- respected_sla %>%
    left_join(
      bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)
    ) %>%
    left_join(node_levels %>% rename(winner = name)) %>%
    mutate(y = count.acceptable) %>%
    {
      .
    }

  # print(respected_sla %>% ungroup() %>% select(docker_fn_name) %>% distinct())
  p <- ggplot(
    data = df,
    aes(x = factor(level_value), y = y, color = docker_fn_name, alpha = 1)
  ) +
    facet_grid(rows = vars(pipeline)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    scale_y_continuous(labels = scales::percent) +
    scale_x_continuous(labels = scales::percent) +
    labs(
      x = "Placement method",
      y = "Mean satisfaction rate"
    ) +
    geom_quasirandom(method = "tukey", alpha = .2)

  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
  }
  return(p)
}
