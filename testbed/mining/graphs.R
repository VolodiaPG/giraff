output_latency <- function(latency) {
    fig(15, 15)
    latency %>%
        filter(field == "raw") %>%
        adjust_timestamps() %>%
        rename(source = instance, destination = destination_name, latency_value = value) %>%
        select(timestamp, source, destination, folder, latency_value, diff) %>%
        mutate(sorted_interaction = pmap_chr(list(source, destination), ~ paste(sort(c(...)), collapse = "_"))) %>%
        ggplot(aes(x = sorted_interaction, y = latency_value, color = (interaction(source, destination, sep = "_") == sorted_interaction), group = interaction(source, destination))) +
        facet_grid(cols = vars(folder)) +
        theme(axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)) +
        geom_boxplot() +
        theme(legend.position = "none")
}

output_loss <- function(latency) {
    latency %>%
        filter(field == "raw_packet_loss") %>%
        adjust_timestamps() %>%
        rename(source = instance, destination = destination_name) %>%
        select(timestamp, source, destination, folder, value) %>%
        mutate(sorted_interaction = pmap_chr(list(source, destination), ~ paste(sort(c(...)), collapse = "_"))) %>%
        ggplot(aes(x = sorted_interaction, y = value, color = (interaction(source, destination, sep = "_") == sorted_interaction), group = interaction(source, destination))) +
        facet_grid(cols = vars(folder)) +
        theme(axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)) +
        geom_boxplot() +
        theme(legend.position = "none")
}

output_gif <- memoised(function(raw.cpu.observed_from_fog_node, bids_won_function) {
    data <- latency %>%
        filter(field == "raw") %>%
        adjust_timestamps() %>%
        ungroup() %>%
        rename(source = instance, destination = destination_name, latency_value = value) %>%
        select(timestamp, source, destination, folder, latency_value, diff) %>%
        smooth_timestamps() %>%
        group_by(timestamp_group, source, destination, folder) %>%
        summarise(latency_value = mean(latency_value), diff = mean(diff)) %>%
        rename(timestamp = timestamp_group) %>%
        ungroup() %>%
        {
            .
        }


    nodes <- load_csv("provisioned_functions.csv") %>%
        prepare() %>%
        adjust_timestamps() %>%
        group_by(timestamp, folder, instance) %>%
        mutate(provisioned = ifelse(value == 0, -1, value)) %>%
        summarise(provisioned = sum(provisioned), value = sum(value)) %>%
        group_by(folder, instance) %>%
        arrange(timestamp, .by_group = TRUE) %>%
        mutate(provisioned = lag(cumsum(provisioned), default = 0), total_provisioned = lag(cumsum(value), default = 0)) %>%
        rename(source = instance) %>%
        select(source, timestamp, folder, provisioned, total_provisioned) %>%
        smooth_timestamps() %>%
        group_by(timestamp_group, folder, source) %>%
        summarise(provisioned = last(provisioned), total_provisioned = last(total_provisioned)) %>%
        rename(timestamp = timestamp_group) %>%
        ungroup() %>%
        {
            .
        }

    cpu <- raw.cpu.observed_from_fog_node %>%
        adjust_timestamps() %>%
        rename(source = instance) %>%
        select(timestamp, source, folder, usage) %>%
        smooth_timestamps() %>%
        group_by(timestamp_group, folder, source) %>%
        summarise(usage = mean(usage)) %>%
        rename(timestamp = timestamp_group) %>%
        ungroup()

    gif.apdex.raw <- load_csv("proxy.csv") %>%
        prepare() %>%
        adjust_timestamps() %>%
        adjust_timestamps(var_name = "value_raw") %>%
        mutate(value = value_raw) %>%
        rename(function_name = tags) %>%
        extract_function_name_info() %>%
        smooth_timestamps() %>%
        inner_join(bids_won_function %>% select(sla_id, function_name, metric_group, metric_group_group, folder, winner), by = c("sla_id", "function_name", "folder", "metric_group_group", "metric_group")) %>%
        rename(measured_latency = value, source = winner) %>%
        group_by(timestamp_group, sla_id, folder, source) %>%
        summarise(satisfied_count = sum(measured_latency <= latency), total = n()) %>%
        mutate(apdex = satisfied_count / total)

    gif.apdex.by_node <- gif.apdex.raw %>%
        group_by(timestamp_group, folder, source) %>%
        summarise(apdex = mean(apdex)) %>%
        rename(timestamp = timestamp_group) %>%
        ungroup()

    all_combinations <- data %>%
        select(folder, source, destination) %>%
        distinct() %>%
        full_join(
            data %>%
                select(timestamp) %>%
                distinct() %>%
                full_join(nodes %>%
                    select(timestamp) %>%
                    distinct(), by = "timestamp") %>%
                full_join(gif.apdex.by_node %>%
                    select(timestamp) %>%
                    distinct(), by = "timestamp") %>%
                full_join(cpu %>%
                    select(timestamp) %>%
                    distinct(), by = "timestamp"),
            by = character()
        )


    data <- all_combinations %>%
        full_join(data, by = c("timestamp", "folder", "source", "destination")) %>%
        full_join(nodes, by = c("source", "timestamp", "folder")) %>%
        full_join(gif.apdex.by_node, by = c("source", "timestamp", "folder")) %>%
        full_join(cpu, by = c("source", "timestamp", "folder")) %>%
        group_by(folder, source, destination) %>%
        arrange(timestamp, .by_group = TRUE) %>%
        fill(diff, provisioned, total_provisioned) %>%
        ungroup() %>%
        {
            .
        }

    globally_provisioned <- data %>%
        select(source, folder, timestamp, total_provisioned) %>%
        distinct() %>%
        group_by(folder, timestamp) %>%
        summarise(globally_provisioned = sum(total_provisioned)) %>%
        ungroup()

    data <- data %>%
        filter(source != destination) %>%
        select(source, destination, everything()) %>%
        rename(from = source, to = destination)

    data_grouped <- data %>%
        group_by(folder) %>%
        group_split()

    animations <- lapply(data_grouped, FUN = create_plot)
    return(animations)
})

output_jains <- function(earnings.jains.plot.data.raw) {
    plots.jains.w <- GRAPH_ONE_COLUMN_WIDTH
    plots.jains.h <- GRAPH_ONE_COLUMN_HEIGHT
    plots.jains.caption <- "Jain's index at different ratio of low level latencies"
    fig(plots.jains.w, plots.jains.h)

    my_comparisons <- combn(unique(earnings.jains.plot.data.raw$`Placement method`), 2)
    my_comparisons <- apply(my_comparisons, 2, list)
    my_comparisons <- lapply(my_comparisons, unlist)
    fig(10, 10)
    plots.jains <- earnings.jains.plot.data.raw %>%
        ggplot(aes(alpha = 1, x = `Placement method`, y = score, fill = `Placement method`, color = `Placement method`)) +
        # facet_grid(cols = vars(sprintf("%.1f%% low-latency ƒ", ratio_func_ll * 100))) +
        geom_hline(yintercept = max(earnings.jains.plot.data.raw$worst_case), color = "black") +
        annotate("text", x = "\footnotesize{Edge\\dash{}furthest}", y = max(earnings.jains.plot.data.raw$worst_case) + .05, label = sprintf("$max(1/n)=%s$", max(earnings.jains.plot.data.raw$worst_case)), color = "black") +
        geom_boxplot() +
        geom_beeswarm() +
        # stat_compare_means(comparisons = my_comparisons, label = "p.signif") +
        stat_anova_test() +
        labs(
            x = "Placement method",
            y = "Jain's index"
        ) +
        scale_alpha_continuous(guide = "none") +
        guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
        theme(
            legend.background = element_rect(
                fill = alpha("white", .7),
                size = 0.2, color = "white"
            ),
            axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
        ) +
        theme(legend.position = "top", legend.box = "vertical") +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(0, -10, -10, -10), )

    plots.jains + labs(title = plots.jains.caption)
    return(plots.jains)
}

output_anova_nb_deployed <- function(plots.nb_deployed.data) {
    df <- plots.nb_deployed.data %>% ungroup()

    fig(10, 10)

    plots.nb_deployed.h <- GRAPH_ONE_COLUMN_HEIGHT
    plots.nb_deployed.w <- GRAPH_ONE_COLUMN_WIDTH
    plots.nb_deployed.caption <- "Ratio of deployed functions"
    # mean_cb <- function(Letters, mean){
    #     return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}",Letters, mean*100))
    # }
    # plots.nb_deployed <- anova_boxplot(p, df , "Placement method", "nb_functions", "nb_nodes_group", mean_cb, c(13))
    # plots.nb_deployed + labs(title = plots.nb_deployed.caption)

    # generate_label_df2 <- function(TUKEY, variable){

    #      # Extract labels and factor levels from Tukey post-hoc
    #      Tukey.levels <- TUKEY[[variable]][,4]
    #      Tukey.labels <- data.frame(multcompLetters(Tukey.levels)['Letters'])

    #      #I need to put the labels in the same order as in the boxplot :
    #      Tukey.labels$toto=rownames(Tukey.labels)
    #      Tukey.labels=Tukey.labels[order(Tukey.labels$toto) , ]
    #      return(Tukey.labels)
    #      }

    outliers <- c()

    df <- df %>%
        rename(value_y = nb_functions) %>%
        rename(class_x = `Placement method`) %>%
        rename(var_facet = nb_nodes_group) %>%
        select(class_x, value_y, var_facet) %>%
        filter(!row_number() %in% outliers) %>%
        arrange(as.factor(var_facet))


    max_yvalue <- max(df$value_y)
    min_yvalue <- min(df$value_y)

    min_mean <- df %>%
        group_by(var_facet, class_x) %>%
        summarise(mean = mean(value_y))
    min_mean <- min(min_mean$mean) / 2
    max_pt <- max(df$value_y)

    ANOVA <- aov(value_y ~ class_x * var_facet, data = df)
    TUKEY <- TukeyHSD(x = ANOVA, conf.level = 0.95)

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

    labels <- generate_label_df(TUKEY, "class_x:var_facet")
    names(labels) <- c("Letters", "cat")
    labels <- labels %>%
        rowwise() %>%
        mutate(cat = strsplit(cat, ":")) %>%
        mutate(class_x = cat[1]) %>%
        mutate(var_facet = cat[2])

    df <- df %>%
        left_join(labels)

    final.text <- df %>%
        group_by(var_facet, class_x, Letters) %>%
        summarise(mean = mean(value_y)) %>%
        mutate(value_y = min_mean) %>%
        arrange(class_x)

    p <- df %>%
        ggplot(aes(x = class_x, y = value_y, alpha = 1, fill = Letters)) +
        facet_grid(cols = vars(factor(var_facet, levels = c("$19 \\le n < 34$", "$112 \\le n \\le 119$")))) +
        # facet_grid(cols = vars(var_facet)) +
        labs(
            x = "Placement method",
            y = "Jain's index"
        ) +
        scale_y_continuous(label = scales::percent) +
        scale_alpha_continuous(guide = "none") +
        labs(
            x = "Placement method",
            y = "% of functions placed",
        ) +
        theme(legend.background = element_rect(
            fill = alpha("white", .7),
            size = 0.2, color = alpha("white", .7)
        )) +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
        theme(axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)) +
        guides(colour = guide_legend(nrow = 1)) +
        theme(legend.position = "none") +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        stat_summary(fun = mean, geom = "col", aes(color = Letters)) +
        geom_beeswarm(aes(color = Letters)) +
        geom_boxplot(aes(color = Letters), outlier.shape = NA) +
        geom_text(data = final.text, alpha = 1, aes(x = class_x, y = min_mean, label = sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100)))


    sumup.F <- summary(ANOVA)[[1]][["F value"]][1]
    sumup.p <- summary(ANOVA)[[1]][["Pr(>F)"]][1]
    sumup.p <- case_when(
        sumup.p < 0.001 ~ "$p<0.001$",
        sumup.p < 0.01 ~ "$p<0.01$",
        sumup.p < 0.05 ~ "$p<0.05$",
        TRUE ~ "$p$ is ns"
    )

    p <- p +
        geom_text(data = final.text[2, ] %>% mutate(value_y = max_yvalue), aes(x = class_x, y = value_y), color = "black", label = sprintf("\\footnotesize{Anova $F=%.1f$, %s}", sumup.F, sumup.p))

    return(p)
}

output_sla_plot <- function(respected_sla, bids_won_function, node_levels) {
    compute <- function() {
        df <- respected_sla %>%
            #left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
            left_join(bids_won_function %>% ungroup() %>% select(function_name, winner, folder, sla_id) %>% rename(winner_prev = winner, prev_sla = sla_id, prev_function_name = function_name)) %>%
            # left_join(node_levels %>% rename(winner = name)) %>%
            # mutate(docker_fn_name = paste0("fn_", docker_fn_name, sep = "")) %>%
            #mutate(prev_function = prev_function_name) %>%
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

output_respected_sla_plot <- function(respected_sla, bids_won_function, node_levels) {
    compute <- function() {
        df <- respected_sla %>%
            left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
            left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id) %>% rename(winner_prev = winner, prev_sla = sla_id)) %>%
            left_join(node_levels %>% mutate(level = paste0(level, " (", level_value, ")", sep="")) %>% select(name, folder, level) %>% rename(winner = name)) %>%
            left_join(node_levels %>% mutate(level = paste0(level, " (", level_value, ")", sep ="")) %>% select(name, folder, level) %>% rename(winner_prev = name, level_prev = level)) %>%
#            mutate(sla_id = if_else(acceptable_chained == total, docker_fn_name, sla_id)) %>%
#            mutate(prev_sla = if_else(acceptable_chained == total, prev_function, prev_sla)) %>%
            mutate(level_docker = paste0(level, docker_fn_name, sep = "")) %>%
            mutate(level_prev_value = level_prev) %>%
            mutate(level_prev = paste0(level_prev, prev_function, sep = "")) %>%
            ungroup()

        df2 <- df %>%
            ungroup() %>%
            filter(acceptable_chained != total - all_erors)
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
            mutate(source = prev_function) %>%
            mutate(target = level_docker) %>%
            mutate(value = acceptable_chained) %>%
            mutate(name_source = prev_function) %>%
            mutate(name_target = level)
        links <- df1 %>%
            mutate(source = level_docker) %>%
            mutate(target = docker_fn_name) %>%
            mutate(value = acceptable_chained) %>%
            mutate(name_source = level) %>%
            mutate(name_target = docker_fn_name) %>%
            full_join(links)
        links <- df2 %>%
            mutate(source = prev_function) %>%
            mutate(target = level_docker) %>%
            mutate(value = acceptable_chained) %>%
            mutate(name_source = prev_function) %>%
            mutate(name_target = level) %>%
            full_join(links)
        links <- df2 %>%
            mutate(source = level_docker) %>%
            mutate(target = sla_id) %>%
            mutate(value = acceptable_chained) %>%
            mutate(name_source = level) %>%
            mutate(name_target = docker_fn_name) %>%
            full_join(links)
        links <- df3 %>%
            mutate(source = prev_sla) %>%
            mutate(target = level_docker) %>%
            mutate(value = acceptable_chained) %>%
            mutate(name_source = prev_function) %>%
            mutate(name_target = level) %>%
            full_join(links)


        links <- df3 %>%
            mutate(source = prev_sla) %>%
            mutate(target = "rejected") %>%
            mutate(name_source = prev_function) %>%
            mutate(value = total - acceptable_chained - all_erors) %>%
            full_join(links)
        links <- df3 %>%
           mutate(source = prev_sla) %>%
           mutate(target = "errored") %>%
           mutate(name_source = prev_function) %>%
           mutate(value = all_erors) %>%
           full_join(links)
        links <- df1 %>%
           mutate(source = prev_function) %>%
           mutate(target = "errored") %>%
           mutate(name_source = prev_function) %>%
           mutate(value = all_erors) %>%
           full_join(links)
        
        return(links)
    }

    return(do_sankey(compute))
}

output_respected_data_plot <- function(plots.respected_sla.data) {
    df <- plots.respected_sla.data %>%
        group_by(folder, `Placement method`, toto) %>%
        summarise(satisfied_count = mean(count.acceptable)) %>%
        ungroup()

    p <- ggplot(data = df, aes(alpha = 1)) +
        #  facet_grid(~var_facet) +
        theme(legend.background = element_rect(
            fill = alpha("white", .7),
            size = 0.2, color = alpha("white", .7)
        )) +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
        theme(axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)) +
        theme(legend.position = "none") +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        scale_y_continuous(labels = scales::percent) +
        labs(
            x = "Placement method",
            y = "Mean satisfaction rate"
        )

    fig(10, 10)
    plots.respected_sla.w <- GRAPH_ONE_COLUMN_WIDTH
    plots.respected_sla.h <- GRAPH_ONE_COLUMN_HEIGHT
    plots.respected_sla.caption <- "Mean satisfaction rate"
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    plots.respected_sla <- anova_boxplot(p, df, "Placement method", "satisfied_count", "toto", mean_cb, c(11))
    plots.respected_sla + labs(title = plots.respected_sla.caption)
    return(plots.respected_sla)
}

output_arrival <- function(respected_sla) {
    df <- respected_sla %>%
      extract_function_name_info() %>%
#        left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
#        left_join(node_levels %>% rename(winner = name)) %>%
#        mutate(y = count.acceptable) %>%
        {
            .
        }

    p <- ggplot(data = df, aes(x = docker_fn_name, y = request_interval, color = docker_fn_name, alpha = 1)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        labs(
            x = "function",
            y = "Inter-arrival of requests (s)"
        ) +
        geom_beeswarm()

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    return(p)
}


output_respected_data_plot_simple <- function(respected_sla, bids_won_function, node_levels) {
    df <- respected_sla %>%
        left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
        left_join(node_levels %>% rename(winner = name)) %>%
        mutate(y = count.acceptable) %>%
        {
            .
        }

    print(respected_sla %>% ungroup() %>% select(docker_fn_name) %>% distinct())
    p <- ggplot(data = df, aes(x = factor(level_value), y = y, color = docker_fn_name, alpha = 1)) +
        facet_grid(rows = vars(pipeline)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        scale_y_continuous(labels = scales::percent) +
        labs(
            x = "Placement method",
            y = "Mean satisfaction rate"
        ) +
        geom_beeswarm()

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    return(p)
}

output_errored_plot_simple <- function(respected_sla, bids_won_function, node_levels) {
    df <- respected_sla %>%
        left_join(bids_won_function %>% ungroup() %>% select(winner, folder, sla_id)) %>%
        left_join(node_levels %>% rename(winner = name)) %>%
        mutate(y = server_errored / total) %>%
        mutate(pipline = pipeline) %>%
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

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    return(p)
}

output_in_flight_time_plot_simple <- function(respected_sla, bids_won_function, node_levels) {
    df <- respected_sla %>%
        mutate(measured_latency = as.numeric(measured_latency)) %>%
        select(-sla_id) %>%
        #left_join(bids_won_function %>% ungroup() %>% select(folder, winner, sla_id) %>% distinct()) %>%
        #left_join(node_levels %>% rename(winner = name)) %>%
        mutate(some_not_acceptable = acceptable + all_erors != total) %>%
        {
            .
        }
    p <- ggplot(data = df, aes(x = prev_function, y = measured_latency, color = some_not_acceptable, alpha = 1)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        # scale_y_continuous(trans = "log10") +
        labs(
            x = "Placement method",
            y = "measured latency (in_flight) (s)"
        ) +
        geom_beeswarm()

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    return(p)
}

output_latency_vs_expected_latency_plot <- function(respected_sla, bids_won_function) {
    df <- respected_sla %>%
        mutate(measured_latency = as.numeric(measured_latency)) %>%
        #left_join(bids_won_function %>% ungroup() %>% select(function_name, folder, sla_id) %>% rename(prev_sla = sla_id, prev_function_name = function_name)) %>%
        ungroup() %>%
        extract_function_name_info() %>%
        mutate(some_not_acceptable = acceptable + all_erors != total) %>%
    mutate(ratio = as.numeric(measured_latency) / as.numeric(latency)) %>%
        {
            .
        }
  p <- ggplot(data = df, aes(x = docker_fn_name, y = ratio, color = interaction(prev_function, docker_fn_name, some_not_acceptable), alpha = 1)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        # scale_y_continuous(trans = "log10") +
        geom_abline(slope=0, intercept = 1) + 
        labs(
            x = "Function",
            y = "measured_latency/latency"
        ) +
        geom_beeswarm()

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    return(p)
}
output_duration_distribution_plot <- function(provisioned_sla) {
    df <- provisioned_sla %>%
        extract_function_name_info()
    p <- ggplot(data = df, aes(x = docker_fn_name, y = duration, color = docker_fn_name, alpha = 1)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        labs(
            x = "Placement method",
            y = "function duration (s)"
        ) +
        geom_beeswarm()

    fig(10, 10)

    return(p)
}
output_latency_distribution_plot <- function(provisioned_sla) {
    df <- provisioned_sla %>%
        extract_function_name_info()
    p <- ggplot(data = df, aes(x = docker_fn_name, y = latency, color = docker_fn_name, alpha = 1)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        labs(
            x = "Placement method",
            y = "function required latency (s)"
        ) +
        geom_beeswarm()

    fig(10, 10)

    return(p)
}

output_request_distribution <- function(respected_sla) {
  df <- respected_sla
    p <- ggplot(data = df, aes(x = total, y = acceptable, color = docker_fn_name, alpha = 1)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        labs(
            x = "function",
            y = "number of requests"
        ) +
        geom_point() +
        geom_line() 

    fig(10, 10)

    return(p)
}


output_ran_for_plot_simple <- function(respected_sla) {
    df <- respected_sla %>%
        mutate(ran_for = as.numeric(ran_for)) %>%
        mutate(some_not_acceptable = acceptable + all_erors != total)

    p <- ggplot(data = df, aes(x = interaction(prev_function, docker_fn_name), y = ran_for, color = some_not_acceptable, alpha = 1)) +
        #  facet_grid(~var_facet) +
        theme(legend.background = element_rect(
            fill = alpha("white", .7),
            size = 0.2, color = alpha("white", .7)
        )) +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
        theme(axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)) +
        # scale_y_continuous(trans = "log10") +
        # theme(legend.position = "none") +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        # scale_y_continuous(labels = scales::percent) +
        labs(
            x = "Placement method",
            y = "mean ran_for (s)"
        ) +
        geom_boxplot()

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    return(p)
}
output_function_latency_plot_simple <- function(respected_sla) {
    df <- respected_sla %>%
        mutate(ran_for = as.numeric(ran_for))

    p <- ggplot(data = df, aes(x = pipeline, y = ran_for, color = docker_fn_name, alpha = 1)) +
        #  facet_grid(~var_facet) +
        theme(legend.background = element_rect(
            fill = alpha("white", .7),
            size = 0.2, color = alpha("white", .7)
        )) +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
        theme(axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)) +
        scale_y_continuous(trans = "log10") +
        # theme(legend.position = "none") +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        # scale_y_continuous(labels = scales::percent) +
        labs(
            x = "Placement method",
            y = "mean ran_for (s)"
        ) +
        geom_beeswarm()

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f%%$}", Letters, mean * 100))
    }
    return(p)
}

output_jains_index_plot <- function(earnings.jains.plot.data.raw) {
    df <- earnings.jains.plot.data.raw %>%
        mutate(toto = "toto") %>%
        ungroup()
    p <- ggplot(data = df, aes(alpha = 1)) +
        labs(
            x = "Placement method",
            y = "Jain's index"
        ) +
        scale_alpha_continuous(guide = "none") +
        guides(color = guide_legend(nrow = 1), shape = guide_legend(nrow = 1), size = guide_legend(nrow = 1)) +
        theme(
            legend.background = element_rect(
                fill = alpha("white", .7),
                size = 0.2, color = "white"
            ),
            axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
        ) +
        theme(legend.position = "none") +
        # theme(legend.position = "top", legend.box = "vertical") +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(0, -10, -10, -10), )

    fig(10, 10)
    plots.jains.w <- GRAPH_ONE_COLUMN_WIDTH
    plots.jains.h <- GRAPH_ONE_COLUMN_HEIGHT
    plots.jains.caption <- "Jain's index at different ratio of low level latencies"
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f$}", Letters, mean))
    }
    plots.jains <- anova_boxplot(p, df, "Placement method", "score", "toto", mean_cb)
    plots.jains + labs(title = plots.jains.caption)
    return(plots.jains)
}

output_mean_time_to_deploy <- function(raw.deployment_times) {
    df <- raw.deployment_times %>%
        group_by(folder, metric_group) %>%
        summarise(value = mean(value)) %>%
        correct_names() %>%
        mutate(group = "sdlkfjh") %>%
        ungroup()

    p <- ggplot(data = df, aes(alpha = 1)) +
        #  facet_grid(~var_facet) +
        theme(legend.position = "none") +
        scale_alpha_continuous(guide = "none") +
        labs(
            y = "Mean time to deploy (s)",
            x = "Placement method",
        ) +
        theme(
            legend.background = element_rect(
                fill = alpha("white", .7),
                size = 0.2, color = alpha("white", .7)
            ),
            axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
        ) +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
        # theme(legend.position = c(.8, .5)) +
        guides(colour = guide_legend(ncol = 1)) +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T)
    fig(10, 10)
    plots.deploymenttimes.w <- GRAPH_ONE_COLUMN_WIDTH
    plots.deploymenttimes.h <- GRAPH_ONE_COLUMN_HEIGHT
    plots.deploymenttimes.caption <- "Time to find a fog node for a function"
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1fs$}", Letters, mean))
    }
    plots.deploymenttimes <- anova_boxplot(p, df, "Placement method", "value", "group", mean_cb, c(4, 6, 19))
    plots.deploymenttimes + labs(title = plots.deployment_times.caption)
    return(plots.deploymenttimes)
}

output_mean_time_to_deploy_simple <- function(raw.deployment_times) {
    df <- raw.deployment_times

    p <- ggplot(data = df, aes(x = docker_fn_name, y = value, color = folder, alpha = 1)) +
        #  facet_grid(~var_facet) +
        theme(legend.position = "none") +
        scale_alpha_continuous(guide = "none") +
        labs(
            y = "Mean time to deploy (ms)",
            x = "Placement method",
        ) +
        theme(
            legend.background = element_rect(
                fill = alpha("white", .7),
                size = 0.2, color = alpha("white", .7)
            ),
            axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
        ) +
        scale_y_continuous(trans = "log10") +
        theme(legend.spacing.y = unit(0, "cm"), legend.margin = margin(0, 0, 0, 0), legend.box.margin = margin(-10, -10, -10, -10), ) +
        # theme(legend.position = c(.8, .5)) +
        guides(colour = guide_legend(ncol = 1)) +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        geom_beeswarm()
    return(p)
}

output_spending_plot <- function(plots.spending.data) {
    df <- plots.spending.data %>%
        mutate(group = "toto") %>%
        ungroup()

    p <- ggplot(data = df, aes(alpha = 1)) +
        #  facet_grid(~var_facet) +
        theme(legend.position = "none") +
        scale_alpha_continuous(guide = "none") +
        labs(
            y = "Function cost",
            x = "Placement method",
        ) +
        theme(legend.background = element_rect(
            fill = alpha("white", .7),
            size = 0.2, color = alpha("white", .7)
        )) +
        theme(
            legend.spacing.y = unit(0, "cm"),
            legend.margin = margin(0, 0, 0, 0),
            legend.box.margin = margin(-10, -10, -10, -10),
            axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
        ) +
        # scale_x_discrete(guide = guide_axis(n.dodge = 2)) +
        # theme(legend.position = c(.5, .93)) +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        guides(colour = guide_legend(nrow = 1))

    fig(10, 10)
    mean_cb <- function(Letters, mean) {
        return(sprintf("%s\n\\footnotesize{$\\mu=%.1f$}", Letters, mean))
    }
    plots.spending <- anova_boxplot(p, df, "Placement method", "spending", "group", mean_cb)
    plots.spending + labs(title = plots.spending.caption)
    return(plots.spending)
}

output_spending_plot_simple <- function(plots.spending.data) {
    df <- plots.spending.data %>%
        extract_function_name_info()

    p <- ggplot(data = df, aes(x = winner, y = cost,  color = docker_fn_name, alpha = 1)) +
        theme(legend.position = "none") +
        scale_alpha_continuous(guide = "none") +
        labs(
            y = "Function cost",
            x = "Placement method",
        ) +
        theme(legend.background = element_rect(
            fill = alpha("white", .7),
            size = 0.2, color = alpha("white", .7)
        )) +
        theme(
            legend.spacing.y = unit(0, "cm"),
            legend.margin = margin(0, 0, 0, 0),
            legend.box.margin = margin(-10, -10, -10, -10),
            axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
        ) +
        scale_color_viridis(discrete = T) +
        scale_fill_viridis(discrete = T) +
        guides(colour = guide_legend(nrow = 1)) +
        geom_quasirandom(method='tukey',alpha=.2)

    return(p)
}
