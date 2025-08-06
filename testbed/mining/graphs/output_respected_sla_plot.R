output_respected_sla_plot <- function(respected_sla, bids_won_function, node_levels) {
  compute <- function() {
    df <- respected_sla %>%
      left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
      left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id) %>% rename(winner_prev = winner, prev_sla = sla_id)) %>%
      left_join(node_levels %>% mutate(level = paste0(level, " (", level_value, ")", sep = "")) %>% select(name, folder, level) %>% rename(winner = name)) %>%
      left_join(node_levels %>% mutate(level = paste0(level, " (", level_value, ")", sep = "")) %>% select(name, folder, level) %>% rename(winner_prev = name, level_prev = level)) %>%
      #            mutate(sla_id = if_else(acceptable_chained == total, docker_fn_name, sla_id)) %>%
      #            mutate(prev_sla = if_else(acceptable_chained == total, prev_function, prev_sla)) %>%
      mutate(level_docker = paste0(level, docker_fn_name, sep = " ")) %>%
      mutate(level_prev_value = level_prev) %>%
      mutate(level_prev = paste0(level_prev, prev_function, sep = " ")) %>%
      ungroup()

    df2 <- df %>%
      ungroup() %>%
      filter(acceptable_chained != total - all_errors)
    df3 <- df2 %>%
      select(sla_id)
    df2 <- df2 %>%
      select(prev_sla) %>%
      rename(sla_id = prev_sla)

    df1 <- df %>%
      anti_join(df2)
    df2 <- df %>%
      semi_join(df2)
    df3 <- df %>%
      semi_join(df3)

    links <- df1 %>%
      mutate(source = level_prev) %>%
      mutate(target = level_docker) %>%
      mutate(value = acceptable_chained)
    # mutate(name_source = prev_function) %>%
    # mutate(name_target = docker_fn_name)
    # links <- df1 %>%
    #    mutate(source = level_docker) %>%
    #    mutate(target = docker_fn_name) %>%
    #    mutate(value = acceptable_chained) %>%
    #    mutate(name_source = level) %>%
    #    mutate(name_target = docker_fn_name) %>%
    #    full_join(links)
    # links <- df2 %>%
    #    mutate(source = prev_function) %>%
    #    mutate(target = level_docker) %>%
    #    mutate(value = acceptable_chained) %>%
    #    mutate(name_source = prev_function) %>%
    #    mutate(name_target = level) %>%
    #    full_join(links)
    # links <- df2 %>%
    #    mutate(source = level_docker) %>%
    #    mutate(target = sla_id) %>%
    #    mutate(value = acceptable_chained) %>%
    #    mutate(name_source = level) %>%
    #    mutate(name_target = docker_fn_name) %>%
    #    full_join(links)
    # links <- df3 %>%
    #    mutate(source = prev_sla) %>%
    #    mutate(target = level_docker) %>%
    #    mutate(value = acceptable_chained) %>%
    #    mutate(name_source = prev_function) %>%
    #    mutate(name_target = level) %>%
    #    full_join(links)

    links <- df3 %>%
      mutate(source = level_prev) %>%
      mutate(target = "rejected") %>%
      # mutate(name_source = prev_function) %>%
      mutate(value = total - acceptable_chained - all_errors) %>%
      full_join(links)
    # links <- df3 %>%
    #   mutate(source = prev_sla) %>%
    #   mutate(target = "errored") %>%
    #   mutate(name_source = prev_function) %>%
    #   mutate(value = all_errors) %>%
    #   full_join(links)
    links <- df1 %>%
      mutate(source = level_prev) %>%
      mutate(target = paste0("errored", level_docker, sep = " ")) %>%
      mutate(name_target = paste0("errored", docker_fn_name, sep = " ")) %>%
      mutate(value = all_errors) %>%
      full_join(links)

    return(links)
  }

  return(do_sankey(compute))
}