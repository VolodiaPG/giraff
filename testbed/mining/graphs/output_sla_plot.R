output_sla_plot <- function(respected_sla, bids_won_function, node_levels) {
  compute <- function() {
    df <- respected_sla %>%
      # left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
      left_join(bids_won_function %>% ungroup() %>% select(function_name, winner, folder, sla_id) %>% rename(winner_prev = winner, prev_sla = sla_id, prev_function_name = function_name)) %>%
      # left_join(node_levels %>% rename(winner = name)) %>%
      # mutate(docker_fn_name = paste0("fn_", docker_fn_name, sep = "")) %>%
      # mutate(prev_function = prev_function_name) %>%
      ungroup()

    links <- df %>%
      mutate(source = prev_function) %>%
      mutate(target = docker_fn_name) %>%
      mutate(value = service_oked)
    links <- df %>%
      mutate(source = prev_function) %>%
      mutate(target = "5xx") %>%
      mutate(value = service_server_errored) %>%
      full_join(links)
    links <- df %>%
      mutate(source = prev_function) %>%
      mutate(target = "4xx") %>%
      mutate(value = service_errored) %>%
      full_join(links)
    links <- df %>%
      mutate(source = prev_function) %>%
      mutate(target = "408") %>%
      mutate(value = service_timeouted) %>%
      full_join(links)
    # 404 = no tag so we need the prev function name
    links <- df %>%
      mutate(source = prev_function) %>%
      mutate(target = "404") %>%
      mutate(value = service_not_found) %>%
      full_join(links)
    return(links)
  }

  return(do_sankey(compute))
}