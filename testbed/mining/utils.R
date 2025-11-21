# All this is implemented (plus bugfixes!) in the ggnewscale package:
# https://github.com/eliocamp/ggnewscale
# If you have any issues, I prefer it if you send them as issues here:
# https://github.com/eliocamp/ggnewscale/issues

#' Allows to add another scale
#'
#' @param new_aes character with the aesthetic for which new scales will be
#' created
#'

new_scale <- function(new_aes) {
  structure(ggplot2::standardise_aes_names(new_aes), class = "new_aes")
}

#' Convenient functions
new_scale_fill <- function() {
  new_scale("fill")
}

new_scale_color <- function() {
  new_scale("colour")
}

new_scale_colour <- function() {
  new_scale("colour")
}

new_scale_alpha <- function() {
  new_scale("alpha")
}

new_scale_y <- function() {
  new_scale("y")
}

#' Special behaviour of the "+" for adding a `new_aes` object
#' It changes the name of the aesthethic for the previous layers, appending
#' "_new" to them.
ggplot_add.new_aes <- function(object, plot, object_name) {
  plot$layers <- lapply(plot$layers, bump_aes, new_aes = object)
  plot$scales$scales <- lapply(plot$scales$scales, bump_aes, new_aes = object)
  plot$labels <- bump_aes(plot$labels, new_aes = object)
  plot
}


bump_aes <- function(layer, new_aes) {
  UseMethod("bump_aes")
}

bump_aes.Scale <- function(layer, new_aes) {
  old_aes <- layer$aesthetics[remove_new(layer$aesthetics) %in% new_aes]
  new_aes <- paste0(old_aes, "_new")

  layer$aesthetics[layer$aesthetics %in% old_aes] <- new_aes

  if (is.character(layer$guide)) {
    layer$guide <- match.fun(paste("guide_", layer$guide, sep = ""))()
  }
  layer$guide$available_aes[layer$guide$available_aes %in% old_aes] <- new_aes
  layer
}

bump_aes.Layer <- function(layer, new_aes) {
  original_aes <- new_aes

  old_aes <- names(layer$mapping)[remove_new(names(layer$mapping)) %in% new_aes]
  new_aes <- paste0(old_aes, "_new")

  old_geom <- layer$geom

  old_setup <- old_geom$handle_na
  new_setup <- function(self, data, params) {
    colnames(data)[colnames(data) %in% new_aes] <- original_aes
    old_setup(data, params)
  }

  new_geom <- ggplot2::ggproto(
    paste0("New", class(old_geom)[1]),
    old_geom,
    handle_na = new_setup
  )

  new_geom$default_aes <- change_name(new_geom$default_aes, old_aes, new_aes)
  new_geom$non_missing_aes <- change_name(
    new_geom$non_missing_aes,
    old_aes,
    new_aes
  )
  new_geom$required_aes <- change_name(new_geom$required_aes, old_aes, new_aes)
  new_geom$optional_aes <- change_name(new_geom$optional_aes, old_aes, new_aes)

  layer$geom <- new_geom

  old_stat <- layer$stat

  old_setup2 <- old_stat$handle_na
  new_setup <- function(self, data, params) {
    colnames(data)[colnames(data) %in% new_aes] <- original_aes
    old_setup2(data, params)
  }

  new_stat <- ggplot2::ggproto(
    paste0("New", class(old_stat)[1]),
    old_stat,
    handle_na = new_setup
  )

  new_stat$default_aes <- change_name(new_stat$default_aes, old_aes, new_aes)
  new_stat$non_missing_aes <- change_name(
    new_stat$non_missing_aes,
    old_aes,
    new_aes
  )
  new_stat$required_aes <- change_name(new_stat$required_aes, old_aes, new_aes)
  new_stat$optional_aes <- change_name(new_stat$optional_aes, old_aes, new_aes)

  layer$stat <- new_stat

  layer$mapping <- change_name(layer$mapping, old_aes, new_aes)
  layer
}

bump_aes.list <- function(layer, new_aes) {
  old_aes <- names(layer)[remove_new(names(layer)) %in% new_aes]
  new_aes <- paste0(old_aes, "_new")

  names(layer)[names(layer) %in% old_aes] <- new_aes
  layer
}

change_name <- function(list, old, new) {
  UseMethod("change_name")
}

change_name.character <- function(list, old, new) {
  list[list %in% old] <- new
  list
}

change_name.default <- function(list, old, new) {
  nam <- names(list)
  nam[nam %in% old] <- new
  names(list) <- nam
  list
}

change_name.NULL <- function(list, old, new) {
  NULL
}

remove_new <- function(aes) {
  stringi::stri_replace_all(aes, "", regex = "(_new)*")
}

fig <- function(width, heigth) {
  options(repr.plot.width = width, repr.plot.height = heigth)
}

center_reduction <- function(data, colvar) {
  colvar <- rlang::sym(colvar)
  data %>%
    inner_join(data %>% summarise(mean = mean(!!colvar), sd = sd(!!colvar))) %>%
    mutate(!!colvar := (!!colvar - mean) / sd) %>%
    select(-c("sd", "mean"))
}

# All this is implemented (plus bugfixes!) in the ggnewscale package:
# https://github.com/eliocamp/ggnewscale
# If you have any issues, I prefer it if you send them as issues here:
# https://github.com/eliocamp/ggnewscale/issues

#' Allows to add another scale
#'
#' @param new_aes character with the aesthetic for which new scales will be
#' created
#'
new_scale <- function(new_aes) {
  structure(ggplot2::standardise_aes_names(new_aes), class = "new_aes")
}

#' Convenient functions
new_scale_fill <- function() {
  new_scale("fill")
}

new_scale_color <- function() {
  new_scale("colour")
}

new_scale_colour <- function() {
  new_scale("colour")
}

new_scale_alpha <- function() {
  new_scale("alpha")
}

new_scale_y <- function() {
  new_scale("y")
}

#' Special behaviour of the "+" for adding a `new_aes` object
#' It changes the name of the aesthethic for the previous layers, appending
#' "_new" to them.
ggplot_add.new_aes <- function(object, plot, object_name) {
  plot$layers <- lapply(plot$layers, bump_aes, new_aes = object)
  plot$scales$scales <- lapply(plot$scales$scales, bump_aes, new_aes = object)
  plot$labels <- bump_aes(plot$labels, new_aes = object)
  plot
}


bump_aes <- function(layer, new_aes) {
  UseMethod("bump_aes")
}

bump_aes.Scale <- function(layer, new_aes) {
  old_aes <- layer$aesthetics[remove_new(layer$aesthetics) %in% new_aes]
  new_aes <- paste0(old_aes, "_new")

  layer$aesthetics[layer$aesthetics %in% old_aes] <- new_aes

  if (is.character(layer$guide)) {
    layer$guide <- match.fun(paste("guide_", layer$guide, sep = ""))()
  }
  layer$guide$available_aes[layer$guide$available_aes %in% old_aes] <- new_aes
  layer
}

bump_aes.Layer <- function(layer, new_aes) {
  original_aes <- new_aes

  old_aes <- names(layer$mapping)[remove_new(names(layer$mapping)) %in% new_aes]
  new_aes <- paste0(old_aes, "_new")

  old_geom <- layer$geom

  old_setup <- old_geom$handle_na
  new_setup <- function(self, data, params) {
    colnames(data)[colnames(data) %in% new_aes] <- original_aes
    old_setup(data, params)
  }

  new_geom <- ggplot2::ggproto(
    paste0("New", class(old_geom)[1]),
    old_geom,
    handle_na = new_setup
  )

  new_geom$default_aes <- change_name(new_geom$default_aes, old_aes, new_aes)
  new_geom$non_missing_aes <- change_name(
    new_geom$non_missing_aes,
    old_aes,
    new_aes
  )
  new_geom$required_aes <- change_name(new_geom$required_aes, old_aes, new_aes)
  new_geom$optional_aes <- change_name(new_geom$optional_aes, old_aes, new_aes)

  layer$geom <- new_geom

  old_stat <- layer$stat

  old_setup2 <- old_stat$handle_na
  new_setup <- function(self, data, params) {
    colnames(data)[colnames(data) %in% new_aes] <- original_aes
    old_setup2(data, params)
  }

  new_stat <- ggplot2::ggproto(
    paste0("New", class(old_stat)[1]),
    old_stat,
    handle_na = new_setup
  )

  new_stat$default_aes <- change_name(new_stat$default_aes, old_aes, new_aes)
  new_stat$non_missing_aes <- change_name(
    new_stat$non_missing_aes,
    old_aes,
    new_aes
  )
  new_stat$required_aes <- change_name(new_stat$required_aes, old_aes, new_aes)
  new_stat$optional_aes <- change_name(new_stat$optional_aes, old_aes, new_aes)

  layer$stat <- new_stat

  layer$mapping <- change_name(layer$mapping, old_aes, new_aes)
  layer
}

bump_aes.list <- function(layer, new_aes) {
  old_aes <- names(layer)[remove_new(names(layer)) %in% new_aes]
  new_aes <- paste0(old_aes, "_new")

  names(layer)[names(layer) %in% old_aes] <- new_aes
  layer
}

change_name <- function(list, old, new) {
  UseMethod("change_name")
}

change_name.character <- function(list, old, new) {
  list[list %in% old] <- new
  list
}

change_name.default <- function(list, old, new) {
  nam <- names(list)
  nam[nam %in% old] <- new
  names(list) <- nam
  list
}

change_name.NULL <- function(list, old, new) {
  NULL
}

remove_new <- function(aes) {
  stringi::stri_replace_all(aes, "", regex = "(_new)*")
}

correct_names <- function(x) {
  return(
    x %>%
      mutate(
        placement_method = case_when(
          placement_method == "auctionno_complication" ~
            "\\footnotesize{\\anon{GIRAFF}}",
          placement_method == "auctionreduction" ~
            "\\footnotesize{\\anon{GIRAFF} (reduction)}",
          placement_method == "edge_wardno_complication" ~
            "\\footnotesize{Edge ward}",
          placement_method == "edge_firstno_complication" ~
            "\\footnotesize{Edge first}",
          placement_method == "edge_furthestno_complication" ~
            "\\footnotesize{Edge furthest}",
          placement_method == "mincpurandomno_complication" ~
            "\\footnotesize{MinCPU random}",
          TRUE ~ paste0("\\footnotesize{", placement_method, " (raw)}")
        )
      ) %>%
      mutate(
        placement_method = factor(
          placement_method,
          levels = c(
            "\\footnotesize{\\anon{GIRAFF}}",
            "\\footnotesize{\\anon{GIRAFF} (reduction)}",
            "\\footnotesize{Edge ward}",
            "\\footnotesize{Edge first}",
            "\\footnotesize{Edge furthest}",
            "\\footnotesize{MinCPU random}"
          )
        )
      )
  )
}

fig(20, 20)

adjust_timestamps <- function(
  x,
  var_name = "timestamp",
  reference = "timestamp"
) {
  # Careful where we put this, as the first measurement may not be the same accross all of the combined values for the same folder

  var_sym <- ensym(var_name)
  ref_sym <- ensym(reference)

  minvalue <- x %>%
    group_by(folder) %>%
    summarise(minvalue = min({{ ref_sym }})) %>%
    ungroup()

  return(
    x %>%
      inner_join(minvalue, by = c("folder")) %>%
      mutate({{ var_sym }} := {{ var_sym }} - minvalue) %>%
      select(-minvalue)
  )
}

prepare <- function(x) {
  return(
    x %>%
      rename(timestamp = "_time") %>%
      rename(field = "_field") %>%
      rename(value_raw = "_value") %>%
      mutate(value = as.numeric(value_raw)) %>%
      # filter (timestamp != "_time") %>% # TODO remove this fix, it is here bnecause I forgot to remove the headers each time i concatenated the different influx outputs
      {
        .
      }
  )
}
prepare_convert <- function(x) {
  return(
    x %>%
      mutate(instance = to_snake_case(instance)) %>%
      {
        .
      }
  )
}

extract_context <- function(x) {
  # The first element is the input string
  info <- stringr::str_match(x$metric_group, "(.+)-(.+)-(.+)")
  info2 <- stringr::str_match(
    x$folder,
    ".+\\.env_([0-9]+_[0-9]+)-.*-\\.env\\.([0-9]+)-\\.env\\.live\\.([0-9]+)_.+"
  )
  return(
    x %>%
      ungroup() %>%
      mutate(placement_method = paste0(info[, 2], info[, 4])) %>%
      mutate(economic_method = paste0(info[, 3], info[, 4])) %>%
      mutate(run = info2[, 2]) %>%
      mutate(env = info2[, 3]) %>%
      mutate(env_live = info2[, 4])
  )
}


extract_function_name_info <- function(x) {
  # The first element is the input string
  info <- stringr::str_match(
    x$function_name,
    "(.+)-i([0-9]+)-c([0-9]+)-m([0-9]+)-l([0-9]+)-a([0-9]+)-r([0-9]+)-d([0-9]+)"
  )
  return(
    x %>%
      ungroup() %>%
      mutate(docker_fn_name = info[, 2]) %>%
      mutate(
        docker_fn_name = ifelse(
          is.na(docker_fn_name),
          function_name,
          docker_fn_name
        )
      ) %>%
      mutate(function_index = info[, 3]) %>%
      mutate(cpu = as.numeric(info[, 4])) %>%
      mutate(mem = as.numeric(info[, 5])) %>%
      mutate(
        latency = as.difftime(as.numeric(info[, 6]) / 1000, units = "secs")
      ) %>%
      mutate(
        arrival = as.difftime(as.numeric(info[, 7]) / 1000, units = "secs")
      ) %>%
      mutate(
        request_interval = as.difftime(
          as.numeric(info[, 8]) / 1000,
          units = "secs"
        )
      ) %>%
      mutate(
        duration = as.difftime(as.numeric(info[, 9]) / 1000, units = "secs")
      )
  )
}

extract_functions_pipeline <- function(x) {
  return(
    x %>%
      group_by(folder, metric_group, metric_group_group, req_id) %>%
      arrange(timestamp) %>%
      mutate(pipeline = paste0(docker_fn_name, collapse = "\n")) %>%
      ungroup()
  )
}

load_csv <- function(filename) {
  all_data <- purrr::map_df(
    METRICS_ARKS,
    ~ mutate(
      vroom(
        archive_read(paste(METRICS_PATH, .x, sep = "/"), file = filename),
        progress = FALSE,
        col_types = cols(),
        col_names = TRUE,
        delim = "\t",
        .name_repair = "unique"
      ) %>%
        distinct(),
      folder = tools::file_path_sans_ext(tools::file_path_sans_ext(.x)),
      metric_group = METRICS_GROUP[which(METRICS_ARKS == .x)],
      metric_group_group = METRICS_GROUP_GROUP[which(METRICS_ARKS == .x)]
    )
  )
  Log(all_data)
  return(all_data)
}

load_single_csv <- function(arkfile, filename) {
  archive_path <- paste(METRICS_PATH, arkfile, sep = "/")
  # because tin the tar xz the files are ./file.csv, with the ./ prefix
  filename <- paste0("./", filename)

  if (!file.exists(archive_path)) {
    stop(paste0("archive doesn't exist ", archive))
  }

  tryCatch(
    {
      all_data <-
        vroom(
          archive_read(
            archive_path,
            file = filename
          ),
          progress = FALSE,
          col_types = cols(),
          col_names = TRUE,
          delim = "\t",
          .name_repair = "unique"
        ) %>%
        mutate(
          folder = tools::file_path_sans_ext(tools::file_path_sans_ext(
            arkfile
          )),
          metric_group = METRICS_GROUP[which(METRICS_ARKS == arkfile)],
          metric_group_group = METRICS_GROUP_GROUP[which(
            METRICS_ARKS == arkfile
          )]
        )
    },
    error = function(e) {
      stop(paste(
        "file doesn't exist in archive",
        filename,
        archive_path,
        e,
        sep = " "
      ))
    }
  )

  return(all_data)
}

get_usage <- function(df_raw) {
  max_timestamp <- df_raw %>%
    select(timestamp, instance, folder) %>%
    group_by(instance, folder) %>%
    summarise(total_time = max(timestamp) - min(timestamp))

  df <- df_raw %>%
    filter(field == "initial_allocatable") %>%
    rename(initial_allocatable = value) %>%
    inner_join(
      df_raw %>% filter(field == "used") %>% rename(used = value),
      by = c(
        "timestamp",
        "folder",
        "instance",
        "metric_group",
        "metric_group_group"
      )
    ) %>%
    mutate(usage = used / initial_allocatable) %>%
    select(instance, timestamp, usage, folder, metric_group, metric_group_group)
  stopifnot(nrow(df) * 2 == nrow(df_raw))

  df <- df %>%
    drop_na() %>%
    distinct() %>%
    group_by(instance, folder, metric_group, metric_group_group) %>%
    arrange(timestamp, .by_group = TRUE) %>%
    summarise(
      usage = sum(
        as.numeric(diff(timestamp), units = "secs") * rollmean(usage, 2)
      )
    ) %>%
    inner_join(max_timestamp, by = c("instance", "folder")) %>%
    mutate(usage_ratio = usage / as.numeric(total_time, units = "secs")) %>% # * 100%
    {
      .
    }

  folders <- df %>%
    ungroup() %>%
    select(folder, metric_group, metric_group_group) %>%
    distinct()

  missing_data <- expand.grid(
    instance = node_levels$name,
    folder = folders$folder
  ) %>%
    # inner_join(node_levels %>% mutate(winner = name) %>% select(-name), by = c("winner")) %>%
    inner_join(folders, by = c("folder"))

  df <- df %>%
    ungroup() %>%
    full_join(
      missing_data,
      by = c("instance", "folder", "metric_group", "metric_group_group")
    ) %>%
    ungroup() %>%
    {
      .
    }

  df$usage_ratio[is.na(df$usage_ratio)] <- 0

  df
}

smooth_timestamps <- function(data) {
  return(
    data %>%
      group_by(folder) %>%
      mutate(
        timestamp_group = as.difftime(
          (as.numeric(round(timestamp)) %/% time_interval + 1) * time_interval,
          units = "secs"
        )
      ) %>%
      ungroup()
  )
}

create_plot <- function(data) {
  net <- network(data, directed = TRUE, multiple = TRUE)
  net <- ggnetwork(net)
  name <- as.character(data$folder[1])
  duration <- max(data$timestamp) / time_interval
  # Log(duration)

  nudge_offset_x <- 0.05
  nudge_offset_y <- -0.3 / 2
  nudge_scale_y <- 3

  pggnetwork <-
    ggplot(
      net,
      aes(x = x, y = y, xend = xend, yend = yend)
    ) + # mapping for edges
    geom_edges(
      arrow = arrow(length = unit(3, "pt"), type = "open"), # if directed
      curvature = 0.1,
      aes(size = diff, alpha = latency_value, color = diff)
    ) +
    scale_colour_gradient(low = "green", high = "red", na.value = "grey50") +
    scale_alpha_continuous() +
    new_scale_color() +
    geom_nodes(aes(size = provisioned, color = apdex), alpha = 0.5, ) +
    scale_colour_gradient(
      low = "white",
      high = "darkblue",
      na.value = "grey50"
    ) +
    new_scale_color() +
    geom_nodes(aes(size = provisioned, color = usage), alpha = 1, ) +
    scale_colour_gradient(low = "green", high = "red", na.value = "grey50") +
    new_scale_color() +
    scale_color_manual(values = c("grey40", "grey80")) +
    geom_nodetext(
      aes(label = sprintf("sat: %1.2f", apdex)),
      color = "black",
      nudge_x = nudge_offset_x,
      nudge_y = (nudge_offset_y + 0.3) / nudge_scale_y
    ) +
    # geom_nodetext(aes(label = sprintf("ll: %1.2f", `Low-load.Low-latency`), colour = is.na(`Low-load.Low-latency`), group = vertex.names), nudge_x = nudge_offset_x, nudge_y = (nudge_offset_y + 0.25) / nudge_scale_y) +
    # geom_nodetext(aes(label = sprintf("lh: %1.2f", `Low-load.High-latency`), colour = is.na(`Low-load.High-latency`), group = vertex.names), nudge_x = nudge_offset_x, nudge_y = (nudge_offset_y + 0.2) / nudge_scale_y) +
    # geom_nodetext(aes(label = sprintf("hh: %1.2f", `High-load.High-latency`), colour = is.na(`High-load.High-latency`), group = vertex.names), nudge_x = nudge_offset_x, nudge_y = (nudge_offset_y + 0.15) / nudge_scale_y) +
    # geom_nodetext(aes(label = sprintf("hl: %1.2f", `High-load.Low-latency`), colour = is.na(`High-load.Low-latency`), group = vertex.names), nudge_x = nudge_offset_x, nudge_y = (nudge_offset_y + 0.1) / nudge_scale_y) +
    geom_nodetext(
      aes(label = sprintf("f: %02d/%02d", provisioned, total_provisioned)),
      color = "black",
      nudge_x = nudge_offset_x,
      nudge_y = (nudge_offset_y + 0.05) / nudge_scale_y
    ) +
    geom_nodetext(
      aes(label = sprintf("%s", vertex.names)),
      color = "grey80",
      nudge_x = nudge_offset_x,
      nudge_y = (nudge_offset_y + 0) / nudge_scale_y
    ) +
    # geom_text(x = 0.05, y = 0.95, aes(label = paste0("Globally provisioned:", globally_provisioned)), color = "grey50", check_overlap = TRUE) +
    labs(
      title = "Time: {as.integer(frame_time)}",
      subtitle = sprintf(
        "'%s'\n
     sat: satisfaction rate\n
     ll: low load low lat. ƒ satisfaction rate\n
     lh: low load high lat. ƒ satisfaction rate\n
     hh: high load high lat. ƒ satisfaction rate\n
     hl: high load low lat. ƒ satisfaction rate\n
     f: <current provisioned>/<total provisioned",
        name
      )
    ) +
    transition_time(timestamp) +
    ease_aes("linear") +
    enter_fade() +
    exit_fade() +
    theme_blank()
  # theme(legend.position = "bottom")
  # +
  # facet_grid(cols = vars(folder))

  # out <- ggplot_build(pggnetwork)

  # rows <- max(out$layout$layout$ROW)
  # cols <- max(out$layout$layout$COL)
  # Log(duration)

  anim_save(
    filename = sprintf("out/%s.gif", name),
    animation = pggnetwork,
    renderer = magick_renderer(),
    nframes = duration,
    height = 1600,
    width = 2000
  )
}

jains_index <- function(allocations) {
  num_users <- length(allocations)
  sum_allocations <- sum(allocations)
  sum_square_allocations <- sum(allocations^2)
  index <- (sum_allocations^2) / (num_users * sum_square_allocations)
  return(index)
}

generate_label_df <- function(TUKEY, variable) {
  # Extract labels and factor levels from Tukey post-hoc
  Tukey.levels <- TUKEY[[variable]][, 4]
  Tukey.labels <- data.frame(multcompLetters(Tukey.levels)["Letters"])

  # I need to put the labels in the same order as in the boxplot :
  Tukey.labels$class_x <- rownames(Tukey.labels)
  Tukey.labels <- Tukey.labels[order(Tukey.labels$class_x), ]
  return(Tukey.labels)
}

anova_boxplot <- function(p, df, x, y, facet, mean_cb, outliers = c()) {
  xvar <- rlang::sym(x)
  yvar <- rlang::sym(y)
  facetvar <- rlang::sym(facet)

  df <- df %>%
    rename(value_y = !!y) %>%
    rename(class_x = !!x) %>%
    rename(var_facet = !!facetvar) %>%
    select(class_x, value_y, var_facet)

  max_yvalue <- max(df$value_y)
  min_yvalue <- min(df$value_y)

  min_mean <- df %>%
    group_by(var_facet, class_x) %>%
    summarise(mean = mean(value_y))
  min_mean <- min(min_mean$mean) / 2
  max_pt <- max(df$value_y)

  for (facetk in as.character(unique(df$var_facet))) {
    subdf.raw <- subset(df, var_facet == facetk)
    print(subdf.raw %>% filter(row_number() %in% outliers))
    subdf <- subdf.raw %>%
      select(-var_facet) %>%
      filter(!row_number() %in% outliers)

    ANOVA <- aov(subdf$value_y ~ subdf$class_x)
    TUKEY <- TukeyHSD(x = ANOVA, "subdf$class_x", conf.level = 0.95)

    tryCatch(
      {
        print("Shapiro (p should be ns)")
        # Extract the residuals
        aov_residuals <- residuals(object = ANOVA)
        # Run Shapiro-Wilk test
        print(shapiro.test(x = aov_residuals))
        print("ANOVA")
        print(summary(ANOVA))
        print("TUKEY")
        print(TUKEY)
        print(plot(ANOVA, 1))
        print(plot(ANOVA, 2))
      },
      error = function(e) {
        message("An error occured with the anova pre checks")
        print(e)
      },
      warning = function(e) {
        message("A warning occured with the anova pre checks")
        print(e)
        return(NA)
      }
    )

    display_labels <- FALSE
    final.text <- NULL
    final.box <- tryCatch(
      {
        labels <- generate_label_df(TUKEY, "subdf$class_x")
        names(labels) <- c("Letters", "class_x")
        yvalue <- aggregate(. ~ class_x, data = subdf, quantile, probs = .75)
        display_labels <- summary(ANOVA)[[1]][["Pr(>F)"]][1] <= 0.05
        return(subdf.raw %>% inner_join(labels))
      },
      error = function(e) {
        return(subdf.raw)
      }
    )

    print(final.box)

    if (display_labels) {
      final.text <- merge(labels, yvalue)
      final.text$var_facet <- facetk
      final.text <- final.text %>%
        inner_join(
          subdf %>% group_by(class_x) %>% summarise(mean = mean(value_y))
        )
      p <- p +
        stat_summary(
          data = final.box,
          fun = mean,
          geom = "col",
          mapping = aes(x = class_x, y = value_y, fill = Letters)
        ) +
        geom_beeswarm(
          data = final.box,
          aes(x = class_x, y = value_y, fill = Letters, color = Letters)
        ) +
        geom_boxplot(
          data = final.box,
          aes(x = class_x, y = value_y, fill = Letters, color = Letters),
          outlier.shape = NA
        ) +
        geom_text(
          data = final.text,
          alpha = 1,
          aes(x = class_x, y = min_mean, label = mean_cb(Letters, mean))
        ) # vjust=-1.5, hjust=-.5
    } else {
      p <- p +
        stat_summary(
          data = final.box,
          fun = mean,
          geom = "col",
          mapping = aes(x = class_x, y = value_y)
        ) +
        geom_beeswarm(data = final.box, aes(x = class_x, y = value_y)) +
        geom_boxplot(
          data = final.box,
          aes(x = class_x, y = value_y),
          outlier.shape = NA
        )
      # geom_text(data = final.text,  alpha= 1, aes(x=class_x, y=min_mean, label=mean_cb("\\dash{}", mean) )) #vjust=-1.5, hjust=-.5
    }

    display(head(final.text))

    sumup.F <- summary(ANOVA)[[1]][["F value"]][1]
    sumup.p <- summary(ANOVA)[[1]][["Pr(>F)"]][1]
    sumup.p <- case_when(
      sumup.p < 0.001 ~ "$p<0.001$",
      sumup.p < 0.01 ~ "$p<0.01$",
      sumup.p < 0.05 ~ "$p<0.05$",
      # sumup.p < 0.1 ~ "$p<0.1$",
      TRUE ~ "$p$ is ns"
    )

    if (!is.null(final.text)) {
      final.text <- final.text %>%
        filter(class_x == subdf$class_x[[1]]) %>%
        mutate(y = max_pt)

      p <- p +
        # geom_text(data = final.text,  alpha= 1, aes(x=class_x, y=min_yvalue, label=sprintf("\\footnotesize{$\\mu=%.1f%%$}",mean*100))) +
        # geom_text(data = final.text,  alpha= 1, aes(x=class_x, y=min_yvalue, label=sprintf("%.1f%%",mean*100))) +
        # stat_anova_test(data= final.box, mapping=aes(x=class_x, y=value_y))
        # annotate(data=final.box, geom = "text", color="black", x= subdf$class_x[[1]], y = max_mean * 1.01, label=sprintf("\\footnotesize{Anova $F=%.1f$, %s}", sumup.F, sumup.p))
        geom_text(
          data = final.text,
          aes(x = class_x, y = y * 1.01),
          hjust = 0,
          color = "black",
          label = sprintf(
            "\\footnotesize{Anova $F=%.1f$, %s}",
            sumup.F,
            sumup.p
          )
        )
    }
  }

  return(p)
}

escape_latex_special_chars <- function(text) {
  # Define special characters to escape
  special_chars <- c(
    "%",
    "&",
    "#",
    "_",
    "\\$",
    "\\{",
    "\\}",
    "\\^",
    "\\~"
  )

  # Escape each special character with a backslash
  for (char in special_chars) {
    text <- gsub(
      char,
      paste0("\\", substring(char, nchar(char), nchar(char))),
      text,
      fixed = TRUE
    )
  }

  return(text)
}


tibble_to_latex_tabular <- function(data, file) {
  cat(
    "\\begin{tabular}{",
    paste0(rep("c", ncol(data)), collapse = " "),
    "}\n",
    file = file
  )
  cat("\\hline\n", file = file, append = TRUE)

  # Print column names with escaped special characters
  cat(
    paste0(escape_latex_special_chars(colnames(data)), collapse = " & "),
    " \\\\\n",
    file = file,
    append = TRUE
  )
  cat("\\hline\n", file = file, append = TRUE)

  # Print rows with escaped special characters
  for (i in 1:nrow(data)) {
    cat(
      paste0(
        escape_latex_special_chars(as.character(data[i, ])),
        collapse = " & "
      ),
      " \\\\\n",
      file = file,
      append = TRUE
    )
  }

  cat("\\hline\n", file = file, append = TRUE)
  cat("\\end{tabular}", file = file, append = TRUE)
}

write_multigraphs <- function(name, ...) {
  args <- list(...)
  widget <- browsable(tagList(
    h1(name),
    lapply(args, function(subgroup) {
      graph <- subgroup$graph
      # if (subgroup$type == "ggplot") {
      graph <- ggplotly(graph) %>%
        layout(boxmode = "group")
      # }
      graph$height <- "100vh"
      ret <- div(graph)
      # title <- div(h4(tag))
      return(list(ret))
    })
  ))
  save_html(widget, paste0("out/", name, ".htm"))
}

export_graph <- function(name, ggplot_graph) {
  callback_name <- deparse(substitute(ggplot_graph))
  Log(paste0("Graphing (", name, ") ", callback_name, " ..."))
  ggsave(paste0("out/", name, ".png"), ggplot_graph)

  # Extract subtitle from the ggplot object
  plot_subtitle <- ggplot_graph$labels$subtitle

  # Create plotly object
  p <- ggplotly(ggplot_graph) %>%
    layout(boxmode = "group")

  # Add subtitle to plotly object if it exists
  if (!is.null(plot_subtitle)) {
    p <- p %>%
      layout(
        title = list(
          text = paste0(
            p$x$layout$title$text,
            "<br><sub>",
            plot_subtitle,
            "</sub>"
          ),
          font = list(size = 14)
        )
      )
  }

  htmlwidgets::saveWidget(p, paste0("out/", name, ".htm"), selfcontained = TRUE)
  Log(paste0("Graphing ", name, " ...done"))
  return(list(name = name, graph = ggplot_graph))
}

export_graph_non_ggplot <- function(name, graph) {
  htmlwidgets::saveWidget(
    graph,
    paste0("out/", name, ".htm"),
    selfcontained = TRUE
  )
}

draw <- function(plot, x_in = 3, y_in = 3) {
  grid::grid.newpage()

  # grid::rectGrob(gp = grid::gpar(fill = "gray")) |>
  #   grid::grid.draw()

  # grid::rectGrob() |>
  #   grid::grid.draw()

  grid::viewport() |>
    grid::pushViewport()

  ggplot2::ggplot_build(plot) |>
    ggplot2::ggplot_gtable() |>
    grid::grid.draw()
}

load_tikz <- function() {
  # library(tikzDevice)

  options(tikzDefaultEngine = "xetex")
  options(
    tikzSanitizeCharacters = c(
      "%",
      "&",
      "#",
      "\\$",
      "\\{",
      "\\}",
      "\\^",
      "\\~",
      "ƒ",
      "_"
    ),
    tikzReplacementCharacters = c(
      "\\%",
      "\\&",
      "\\#",
      "\\\\$",
      "\\\\{",
      "\\\\}",
      "\\\\^",
      "\\\\~",
      "$f$",
      "\\_"
    )
  )
  options(
    tikzXelatexPackages = c(
      "\\usepackage{tikz}\n",
      "\\usepackage[active,tightpage,xetex]{preview}\n",
      "\\usepackage{fontspec,xunicode}\n",
      "\\PreviewEnvironment{pgfpicture}\n",
      "\\setlength\\PreviewBorder{0pt}\n",
      "\\newcommand{\\dash}{-}\n",
      "\\newcommand{\\anon}[1]{ANON}\n"
    )
  )
  options(
    tikzMetricPackages = c(
      "\\usepackage[T1]{fontenc}\n",
      "\\usetikzlibrary{calc}\n"
    ),
    tikzUnicodeMetricPackages = c(
      "\\usepackage[T1]{fontenc}\n",
      "\\usetikzlibrary{calc}\n",
      "\\usepackage{fontspec,xunicode}\n"
    )
  )

  tikzDevice::tikzTest()
}

wrap_graph <- function(graph) {
  list(graph = graph)
}

check_legend_position <- function(built) {
  # Build the plot to get all resolved theme elements
  # Get the theme from the built plot
  if (!is.null(built$plot$theme$legend.position)) {
    return(built$plot$theme$legend.position)
  }

  # Fall back to current default theme
  return(theme_get()$legend.position %||% "right")
}


export_graph_tikz <- function(
  plot,
  width,
  height,
  remove_legend = FALSE,
  caption = NULL
) {
  # oldwd <- getwd()
  # setwd(paste0(getwd(), "/out"))
  # on.exit(setwd(oldwd))

  name <- deparse(substitute(plot))
  name <- stringr::str_replace_all(name, "(.*?)_graph", "\\1")
  # Log(name)
  # if (length(find.package("tikzDevice", quiet = TRUE)) == 0) {
  #   warning("tikzDevice package not found. TikZ export skipped.")
  #   Log("tikzDevice package not found. TikZ export skipped.")
  #   return()
  # }

  plot_name <- to_lower_camel_case(name)
  load_tikz()

  # plot_name <- to_lower_camel_case(name)
  plot_graph <- plot$graph
  # tikz_name <- paste0(plot_name, ".tex")
  # caption_name <- paste0(plot_name, "_caption.tex")

  # Extract title, subtitle, and axis labels from the ggplot object
  built_plot <- ggplot2::ggplot_build(plot_graph)
  plot_title <- built_plot$plot$labels$title
  plot_subtitle <- built_plot$plot$labels$subtitle
  plot_x_label <- built_plot$plot$labels$x
  plot_y_label <- built_plot$plot$labels$y

  plot_graph <- plot_graph +
    ggplot2::theme(
      # title = ggplot2::element_blank(),
      # plot.margin = ggplot2::margin(0, 0, 0, 0, "pt"),
      # panel.spacing = ggplot2::unit(0, "pt"),
      # panel.margin = ggplot2::margin(0, 0, 0, 0, "pt"),
    )
  # if (remove_legend) {
  #   plot_graph <- plot_graph + theme(legend.position = "none")
  # } else if (check_legend_position(plot_graph) != "none") {
  plot_graph <- plot_graph +
    theme(
      legend.position = "top",
      legend.box = "vertical",
      legend.margin = ggplot2::margin(),
      legend.box.margin = ggplot2::margin(),
      legend.spacing = ggplot2::unit(0, "pt"),
      legend.justification = c(0, 0.8)
    ) +
    guides(col = guide_legend(nrow = 4, byrow = TRUE))
  # }
  plot_graph <- plot_graph +
    theme(
      axis.title.x = ggplot2::element_text(plot_x_label),
      axis.title.y = ggplot2::element_text(plot_y_label),
      panel.border = ggplot2::element_blank(),
      panel.background = ggplot2::element_blank(),
      axis.line = ggplot2::element_line(color = "black"),
    )

  # Open the file in write mode
  # file_conn <- file(tikz_name, "w")

  # Write the resizebox command
  # tex_width <- 1

  # ggsave(
  #   filename = paste0(plot_name, ".png"),
  #   plot = plot_graph,
  #   width = width,
  #   height = height,
  #   dpi = 300
  # )

  # cat(
  #   # sprintf("\\begin{minipage}{\\columnwidth}\n"),
  #   sprintf("\\resizebox{\\columnwidth}{!}{\n\\footnotesize\n"),
  #   file = file_conn
  # )
  # cat(sprintf("\\tikzsetnextfilename{%s}\n", plot_name), file = file_conn)

  # Capture tikz output
  # capture.output(
  #   file = file_conn,
  #   # append = TRUE,
  tikz(
    filename = paste0("figures/", plot_name, ".tex"),
    console = FALSE,
    width = width,
    height = height,
    standAlone = FALSE,
    sanitize = TRUE,
    verbose = FALSE
  )
  draw(plot_graph, x_in = width, y_in = height)
  graphics.off()
  # dev.off()
  # )
  #
  # Close the resizebox
  # cat("}\n", file = file_conn)
  # cat("\\end{minipage}\n", file = file_conn)

  # Close the file connection
  # close(file_conn)

  # Export caption to a separate file
  # caption_conn <- file(caption_name, "w")
  # plot_name <- gsub("_", "", plot_name)
  # if (!is.null(caption)) {
  #   cat(sprintf("%s\\label{fig:%s}\n", caption, plot_name), file = caption_conn)
  # } else if (!is.null(plot_title) || !is.null(plot_subtitle)) {
  #   caption <- plot_title
  #   if (!is.null(plot_subtitle)) {
  #     caption <- paste1(
  #       caption,
  #       ". \\\\ \\footnotesize\\textcolor{gray}{\\textit{",
  #       plot_subtitle,
  #       "}}"
  #     )
  #   }
  #   cat(sprintf("%s\\label{fig:%s}\n", caption, plot_name), file = caption_conn)
  # }
  # close(caption_conn)
}

merge_and_export_legend <- function(
  dummy_graphs,
  legend_name,
  width,
  height,
  aspect_ratio = 1 / 5
) {
  if (length(find.package("tikzDevice", quiet = TRUE)) == 0) {
    warning("tikzDevice package not found. TikZ export skipped.")
    Log("tikzDevice package not found. TikZ export skipped.")
    return()
  }

  # Extract names and graphs from dummy_graphs
  graph_names <- lapply(dummy_graphs, function(x) x$name)
  graphs <- lapply(dummy_graphs, function(x) x$graph)

  combined_legend <- wrap_plots(graphs) + plot_layout(guides = "collect")
  legend <- cowplot::get_legend(combined_legend)

  # Create a plot with the extracted legend
  legend_plot <- ggplot() +
    theme_void() +
    theme(
      legend.position = "right",
      legend.box.margin = margin(0, 95, 0, 0, "pt"),
      aspect.ratio = aspect_ratio,
      plot.margin = margin(0, 0, 0, 95, "pt")
    ) +
    annotation_custom(grob = legend)

  ggsave("out/legend.png", legend_plot, width = width, height = height)

  Log(paste0("Exporting legend '", legend_name, "' ..."))

  # Create custom captions referencing the LaTeX labels
  custom_captions <- sapply(graph_names, function(name) {
    sprintf("fig:%s", name)
  })
  custom_caption <- paste0(
    "Legend for~\\cref{",
    paste(custom_captions, collapse = ","),
    "} "
  )

  export_graph_tikz(
    list(name = legend_name, graph = legend_plot),
    width,
    height,
    remove_legend = FALSE,
    caption = custom_caption,
    aspect_ratio = aspect_ratio
  )
}

do_sankey <- function(f) {
  links <- f()
  df <- links %>%
    group_by(source, target) %>%
    summarise(value = sum(value, na.rm = TRUE)) %>%
    select(source, target, value) %>%
    ungroup()

  nodes <- df %>%
    ungroup() %>%
    select(target) %>%
    distinct() %>%
    rename(name = target)
  nodes <- df %>%
    ungroup() %>%
    select(source) %>%
    distinct() %>%
    rename(name = source) %>%
    full_join(nodes) %>%
    distinct() %>%
    as.data.frame()

  ii <- function(name) {
    if (is.na(name[1])) {
      return(which(is.na(nodes))[1] - 1)
    } else {
      return(which(nodes$name == name[1])[1] - 1)
    }
  }

  labels <- links %>%
    ungroup() %>%
    rowwise() %>%
    mutate(
      name = if ("name_source" %in% names(.)) {
        coalesce(.data[["name_source"]], source)
      } else {
        source
      }
    ) %>%
    rename(original = source) %>%
    select(original, name) %>%
    distinct()
  labels <- links %>%
    ungroup() %>%
    rowwise() %>%
    mutate(
      name = if ("name_target" %in% names(.)) {
        coalesce(.data[["name_target"]], target)
      } else {
        target
      }
    ) %>%
    rename(original = target) %>%
    select(original, name) %>%
    distinct() %>%
    full_join(labels) %>%
    distinct()
  df <- df %>%
    rowwise() %>%
    mutate(source = ii(source)) %>%
    mutate(target = ii(target)) %>%
    as.data.frame()

  labels <- nodes %>%
    mutate(row_order = row_number()) %>%
    rename(original = name) %>%
    inner_join(labels) %>%
    arrange(row_order) %>%
    select(-row_order)
  # p <- sankeyNetwork(Links = df, Nodes = nodes, Source = "source", Target = "target", Value = "value", NodeID = "name")
  fig <- plot_ly(
    type = "sankey",
    orientation = "h",
    node = list(
      label = labels$name,
      pad = 15,
      thickness = 20,
      line = list(
        color = "black",
        width = 0.5
      )
    ),
    link = df
  )
  fig <- fig %>%
    layout(
      title = "Basic Sankey Diagram",
      font = list(
        size = 10
      )
    )
  return(fig)
}

# https://github.com/r-lib/memoise/issues/79
memoise2 <- function(f, ..., expr_vars = character(0)) {
  f_wrapper <- function(args) {
    return(eval_tidy(expr(f(!!!args))))
  }
  f_wrapper_m <- memoise(f_wrapper, ...)

  function(...) {
    # Capture args as (unevaluated) quosures
    dot_args <- rlang::dots_list(...)
    # print(dot_args)

    # For each arg, if it's in our set of `expr_vars`, extract the expression
    # (and discard the environment); if it's not in that set, then evaluate the
    # quosure.
    dot_args <- mapply(
      dot_args,
      names(dot_args),
      FUN = function(quo, name) {
        if (name %in% expr_vars) {
          get_expr(quo)
        } else {
          eval_tidy(quo)
        }
      },
      SIMPLIFY = FALSE
    )

    # Print out the captured items
    # print(str(dot_args))

    # Call the memoized wrapper function.
    return(f_wrapper_m(dot_args))
  }
}

Log <- function(text, ...) {
  fifo_path <- file.path(getwd(), "logs.fifo")
  if (!file.exists(fifo_path)) {
    stop(
      "FIFO file does not exist. Please create it manually: mkfifo ",
      fifo_path
    )
  }

  # Open the FIFO for writing (append mode)
  # Note: writing to a FIFO in R can be blocking if nothing is reading from it.
  con <- file(fifo_path, open = "a")

  captured_output <- capture.output(print(text))
  msg <- sprintf(
    paste0("\033[32m", as.character(Sys.time()), "\033[39m", ": %s", sep = ""),
    paste(captured_output, collapse = "\n")
  )

  writeLines(msg, con)
  close(con)
}

create_metric_comparison_plot <- function(
  data,
  metric_col,
  group_col,
  value_col,
  node_col,
  title,
  x_label = NULL,
  x_suffix = NULL,
  y_label = NULL,
  y_suffix = "",
  facet_col = NULL,
  se = FALSE
) {
  empty_facet_col <- is.null(facet_col)

  # Convert column names to symbols
  metric_col <- ensym(metric_col)
  group_col <- ensym(group_col)
  value_col <- ensym(value_col)
  node_col <- ensym(node_col)
  if (!empty_facet_col) {
    facet_col <- ensym(facet_col)
  }

  # Prepare the data
  group_vars <- c(quo_name(group_col), quo_name(metric_col), quo_name(node_col))
  if (!empty_facet_col) {
    group_vars <- c(group_vars, quo_name(facet_col))
  }

  # Check if ci_lower and ci_upper exist in the data
  if ("ci_lower" %in% names(data) && "ci_upper" %in% names(data)) {
    group_vars <- c(group_vars, "ci_lower", "ci_upper")
  }

  # Prepare the data
  df <- data %>%
    group_by(across(all_of(group_vars))) %>%
    summarise(
      metric_value = mean(as.numeric(!!value_col)),
      metric_sd = sd(as.numeric(!!value_col)),
      n = n(),
      .groups = "drop"
    ) %>%
    filter(!is.na(metric_value) & is.finite(metric_value))

  if (!("ci_lower" %in% names(data) && "ci_upper" %in% names(data))) {
    df <- df %>%
      mutate(
        se = metric_sd / sqrt(n),
        ci = se * qt((1 - 0.05) / 2 + .5, n - 1),
        ci_lower = metric_value - ci,
        ci_upper = metric_value + ci
      )
  }

  # Create the scatter plot
  p <- ggplot(
    df,
    aes(
      x = !!node_col,
      y = metric_value,
      color = !!metric_col,
      fill = !!metric_col
    )
  ) +
    geom_point(
      alpha = 0.7,
      aes(
        text = sprintf(
          "Placement Method: %s<br>Nodes: %s<br>Value: %.2f%s<br>CI: [%.2f, %.2f]",
          !!metric_col,
          !!node_col,
          metric_value,
          y_suffix,
          ci_lower,
          ci_upper
        )
      )
    ) +
    geom_smooth(
      method = "lm",
      se = se,
      level = 0.95,
      alpha = 0.1,
      show.legend = TRUE
    ) +
    geom_errorbar(
      aes(ymin = ci_lower, ymax = ci_upper, color = !!metric_col),
      width = 0.2,
      alpha = 0.5
    ) +
    scale_y_continuous(labels = scales::number_format(suffix = y_suffix)) +
    scale_x_continuous(labels = scales::number_format(suffix = x_suffix)) +
    coord_cartesian(ylim = c(0, NA)) +
    labs(
      title = title,
      x = x_label,
      y = y_label,
      color = "Placement Method",
      fill = "Placement Method",
      linetype = "Placement Method",
      shape = "Placement Method",
    ) +
    theme(
      legend.position = "right",
    ) +
    scale_color_viridis_d() +
    scale_fill_viridis_d()

  # Add faceting if facet_col is provided
  if (!empty_facet_col) {
    p <- p + facet_wrap(vars(!!facet_col))
  }

  file <- paste0("out/", title, ".txt", sep = "")
  file_conn <- file(file, "w")
  capture.output(
    file = file_conn,
    append = TRUE,
    df %>%
      group_by(!!metric_col, ) %>%
      select(!!metric_col, metric_value) %>%
      dlookr::describe(),
    graphics.off()
  )
  if (!empty_facet_col) {
    capture.output(
      file = file_conn,
      append = TRUE,
      df %>%
        group_by(!!metric_col, !!facet_col) %>%
        select(!!metric_col, !!facet_col, metric_value) %>%
        dlookr::describe(),
      graphics.off()
    )
  }
  close(file_conn)

  return(p)
}

extract_flame_function_name <- function(spans) {
  names <- stringr::str_match(
    spans$span.name,
    "FLAME.Pool.cast \\(Giraff\\.(.*)Backend\\)"
  )

  spans %>%
    mutate(span.name = names[, 2])
}

log10_labels <- function() {
  scales::trans_format("log10", function(x) {
    sapply(x, function(val) {
      return(paste0(
        "$10^{",
        val,
        "}$"
      ))
    })
  })
}
