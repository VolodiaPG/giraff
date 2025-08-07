output_in_flight_time_plot_simple <- function(
  respected_sla,
  bids_won_function,
  node_levels
) {
  df <- respected_sla %>%
    mutate(measured_latency = as.numeric(measured_latency)) %>%
    select(-sla_id) %>%
    # left_join(bids_won_function %>% ungroup() %>% select(folder, winner, sla_id) %>% distinct()) %>%
    # left_join(node_levels %>% rename(winner = name)) %>%
    mutate(some_not_acceptable = acceptable + all_errors != total) %>%
    {
      .
    }
  p <- ggplot(
    data = df,
    aes(
      x = prev_function,
      y = measured_latency,
      color = some_not_acceptable,
      alpha = 1
    )
  ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    # scale_y_continuous(trans = "log10") +
    labs(
      x = "Placement method",
      y = "measured latency (in_flight) (s)"
    ) +
    geom_quasirandom(method = "tukey", alpha = .2)

  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
  }
  return(p)
}
