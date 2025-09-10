flame_func_with_latency <- function(spans, raw_latency) {
  df <- spans %>%
    mutate(source_node = sub("^.*@", "", service.instance.id)) %>%
    select(
      span.name,
      folder,
      service.namespace,
      source_node
    ) %>%
    distinct()

  df2 <- spans %>%
    mutate(function_node = sub("^.*@", "", service.instance.id)) %>%
    select(
      span.name,
      folder,
      service.namespace,
      function_node
    ) %>%
    distinct()

  df_spans_raw <- df %>%
    inner_join(df2)

  latency_with_ip <- raw_latency %>%
    filter(field == "average") %>%
    mutate(
      source_ip = sub(":.*", "", instance_address),
      dest_ip = sub(":.*", "", instance_to)
    ) %>%
    mutate(latency = value_raw / 1e3) %>%
    group_by(folder, source_ip, dest_ip) %>%
    summarise(latency = mean(latency))

  # Prepare span data with node information
  # Helper function to find path latency in tree topology
  find_tree_path_latency <- function(latency_data, source, dest, folder_val) {
    if (source == dest) {
      return(0)
    }

    # Get all edges for this folder
    edges <- latency_data %>% filter(folder == folder_val)

    # Build adjacency list for tree
    adj_list <- list()
    for (i in 1:nrow(edges)) {
      src <- edges$source_ip[i]
      dst <- edges$dest_ip[i]
      lat <- edges$latency[i]

      if (is.null(adj_list[[src]])) {
        adj_list[[src]] <- list()
      }
      if (is.null(adj_list[[dst]])) {
        adj_list[[dst]] <- list()
      }

      adj_list[[src]][[dst]] <- lat
      adj_list[[dst]][[src]] <- lat
    }

    # BFS to find path in tree
    queue <- list(list(node = source, path = c(), total_latency = 0))
    visited <- c()

    while (length(queue) > 0) {
      current <- queue[[1]]
      queue <- queue[-1]

      if (current$node %in% visited) {
        next
      }
      visited <- c(visited, current$node)

      if (current$node == dest) {
        return(current$total_latency)
      }

      if (!is.null(adj_list[[current$node]])) {
        for (neighbor in names(adj_list[[current$node]])) {
          if (!(neighbor %in% visited)) {
            new_latency <- current$total_latency +
              adj_list[[current$node]][[neighbor]]
            queue <- append(
              queue,
              list(list(
                node = neighbor,
                path = c(current$path, current$node),
                total_latency = new_latency
              ))
            )
          }
        }
      }
    }

    return(NA)
  }

  df_spans_raw %>%
    rowwise() %>%
    mutate(
      latency = find_tree_path_latency(
        latency_with_ip,
        source_node,
        function_node,
        folder
      )
    ) %>%
    ungroup() %>%
    filter(!is.na(latency)) %>%
    select(
      span.name,
      folder,
      latency,
      source_node,
      service.namespace,
      function_node
    )
}
