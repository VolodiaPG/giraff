output_arrival <- function(respected_sla) {
  df <- respected_sla %>%
    extract_function_name_info() %>%
    #        left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
    #        left_join(node_levels %>% rename(winner = name)) %>%
    #        mutate(y = count.acceptable) %>%
    {
      .
    }

  p <- ggplot(
    data = df,
    aes(
      x = docker_fn_name,
      y = request_interval,
      color = docker_fn_name,
      alpha = 1
    )
  ) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    labs(
      x = "function",
      y = "Inter-arrival of requests (s)"
    ) +
    geom_quasirandom(method = "tukey", alpha = .2)

  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
  }
  return(p)
}
