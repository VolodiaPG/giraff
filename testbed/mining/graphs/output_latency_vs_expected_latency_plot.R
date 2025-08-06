output_latency_vs_expected_latency_plot <- function(respected_sla, bids_won_function) {
  df <- respected_sla %>%
    mutate(measured_latency = as.numeric(measured_latency)) %>%
    # left_join(bids_won_function %>% ungroup() %>% select(function_name, folder, sla_id) %>% rename(prev_sla = sla_id, prev_function_name = function_name)) %>%
    ungroup() %>%
    extract_function_name_info() %>%
    # mutate(some_not_acceptable = acceptable + all_errors != total) %>%
    mutate(ratio = as.numeric(measured_latency) / as.numeric(latency)) %>%
    {
      .
    }

  p <- ggplot(data = df, aes(x = docker_fn_name, y = ratio, color = interaction(prev_function, docker_fn_name), alpha = 1)) +
    scale_color_viridis(discrete = TRUE) +
    scale_fill_viridis(discrete = TRUE) +
    # scale_y_continuous(trans = "log10") +
    geom_abline(slope = 0, intercept = 1) +
    labs(
      x = "Function",
      y = "measured_latency/latency"
    ) +
    geom_boxplot()
  # geom_quasirandom(method='tukey',alpha=.2)

  mean_cb <- function(Letters, mean) {
    return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
  }
  return(p)
}