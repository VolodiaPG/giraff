output_otel_duration_latency_plot <- function(spans, raw_latency = NULL) {
  df_spans_raw <- spans %>%
    filter(startsWith(span.name, "FLAME") & endsWith(span.name, "...")) %>%
    mutate(span.name = substring(span.name, 1, nchar(span.name) - 3)) %>%
    mutate(source_node = sub("^.*@", "", service.instance.id)) %>%
    select(
      span.name,
      folder,
      out_duration = duration,
      trace_id,
      service.namespace,
      source_node,
      timestamp,
    )

  df_spans_raw2 <- spans %>%
    filter(startsWith(span.name, "...FLAME")) %>%
    mutate(span.name = substring(span.name, 4)) %>%
    mutate(function_node = sub("^.*@", "", service.instance.id)) %>%
    select(
      span.name,
      folder,
      in_duration = duration,
      trace_id,
      service.namespace,
      function_node
    )

  df_spans <- df_spans_raw %>%
    inner_join(df_spans_raw2) %>%
    mutate(duration = out_duration - in_duration) %>%
    select(
      trace_id,
      timestamp,
      span.name,
      folder,
      duration,
      service.namespace,
      source_node,
      function_node
    )

  # df <- df %>%
  #   filter(span.name %in% c("create_machine")) %>%
  #   mutate(duration = end_timestamp - timestamp) %>%
  #   # filter(span.name %in% c("create_machine", "start_processing_requests")) %>%
  #   full_join(df_spans)

  # offs <- as.numeric(factor(df$trace_id))
  # offscale <- (offs - mean(unique(offs))) * 0.005

  # Create correlation matrix if raw_latency is provided
  if (!is.null(raw_latency)) {
    # Extract IP from instance field (format: ip:port)

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
    span_data <- df_spans %>%
      filter(!is.na(duration) & !is.na(source_node) & !is.na(function_node)) %>%
      mutate(duration_numeric = as.numeric(duration))

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

    # Calculate path latencies for each source-function pair
    matched_data <- span_data %>%
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
        trace_id,
        span.name,
        folder,
        duration_numeric,
        latency,
        source_node,
        service.namespace,
        function_node,
        timestamp
      )

    # Create correlation analysis if we have matched data
    if (nrow(matched_data) > 0) {
      # Calculate point sizes based on timestamp within service.namespace lifetime
      matched_data <- matched_data %>%
        left_join(
          spans %>%
            group_by(service.namespace) %>%
            summarise(
              min_timestamp = min(timestamp),
              max_timestamp = max(timestamp)
            ),
          by = "service.namespace"
        ) %>%
        group_by(service.namespace, folder, latency, span.name) %>%
        summarise(
          duration_numeric = mean(duration_numeric)
        )

      # %>%
      # mutate(
      #   timestamp_progress = as.numeric(timestamp - min_timestamp) /
      #     as.numeric(max_timestamp - min_timestamp),
      #   point_size = 4 * timestamp_progress
      # )

      # Calculate correlation coefficient
      correlation_latency_duration <- cor(
        matched_data$duration_numeric,
        matched_data$latency,
        use = "complete.obs"
      )

      # correlation_time_duration <- cor(
      #   matched_data$duration_numeric,
      #   matched_data$timestamp_progress,
      #   use = "complete.obs"
      # )

      # Calculate convex hulls for each group
      hull_data <- matched_data %>%
        group_by(span.name) %>%
        slice(chull(latency, duration_numeric))

      p <- ggplot(
        matched_data,
        aes(
          x = latency,
          y = duration_numeric,
          color = service.namespace
          # size = point_size,
          # shape = source_node == function_node
        )
      ) +
        # geom_line(aes(color = trace_id, group = trace_id)) +
        geom_polygon(
          data = hull_data,
          aes(
            x = latency,
            y = duration_numeric,
            fill = span.name
          ),
          color = NA,
          alpha = 0.2,
          inherit.aes = FALSE
        ) +
        geom_point(alpha = 0.3) +
        scale_size_identity() +
        # geom_smooth(method = "lm", se = TRUE, color = "#BB4444") +
        # scale_y_continuous(trans = "log10") +
        labs(
          title = paste(
            # "Duration vs Latency Correlation (r =",
            # round(correlation_latency_duration, 3),
            # "), Correlation between timestamp and duration: r =",
            # round(correlation_time_duration, 3),
            # "\n",
            "Duration vs Latency of functions\n",
            unique(matched_data$folder)
          ),
          x = "Network Latency (parent_function → next_function)",
          y = "Span Duration (seconds)",
          # subtitle = paste(
          #   "Based on",
          #   nrow(matched_data),
          #   "matched source-function pairs",
          #   "Correlation between timestamp and latency: r=",
          #   round(correlation_time_duration, 3)
          # )
        ) +
        theme_minimal() +
        theme(
          plot.title = element_text(size = 12, hjust = 0.5),
          plot.subtitle = element_text(size = 10, hjust = 0.5),
          axis.title = element_text(size = 10),
          panel.grid.minor = element_blank()
        )

      return(p)
    } else {
      # Return informative plot if no matches found
      return(
        ggplot() +
          annotate(
            "text",
            x = 0.5,
            y = 0.5,
            label = "No matching source-function pairs found\nbetween spans and latency data",
            hjust = 0.5,
            vjust = 0.5,
            size = 4
          ) +
          theme_void()
      )
    }
  }

  # Return empty plot if no correlation data
  ggplot() +
    theme_void()
}
