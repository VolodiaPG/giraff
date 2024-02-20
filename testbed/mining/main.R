init <- function() {
    Sys.setenv(VROOM_TEMP_PATH = "./vroom")
    system("mkdir -p ./vroom")
    system("rm ./vroom/*")

    # To call python from R
    library(archive)
    library(dplyr)
    library(reticulate)
    library(tidyverse)
    library(igraph)
    library(r2r)
    library(formattable)
    library(stringr)
    library(viridis)
    # library(geomtextpath)
    library(cowplot)
    library(scales)
    library(vroom)
    library(zoo)
    library(ggdist)
    library(gghighlight)
    library(ggrepel)
    library(ggbreak)
    library(grid)
    library(lemon)
    library(ggprism)
    library(ggh4x)
    library(ggExtra)
    library(tibbletime)
    library(snakecase)
    library(foreach)
    library(doParallel)
    library(ggside)
    library(ggbeeswarm)
    library(multidplyr)
    library(ggpubr)
    library(Hmisc)
    library(rstatix)
    library(multcompView)
    library(gganimate)

    library(intergraph)
    library(network)
    library(ggnetwork)
    library(treemapify)

    library(memoise)

    library(purrr)
    library(future.apply)
    future::plan("multicore", workers = 20L)

    ggplot2::theme_set(theme_prism())
}

suppressMessages(init())

source("utils.R")
source("config.R")

cd <- cachem::cache_disk(rappdirs::user_cache_dir("R-myapp"), max_size = 5 * 1024 * 1024^2)

if (cd$exists("metrics")) {
    cached <- cd$get("metrics")
    if (!identical(METRICS_ARKS, cached)) {
        cd$reset()
    }
} else {
    cd$reset()
}
cd$set("metrics", METRICS_ARKS)

if (no_memoization) {
    cd$reset()
}

memoised <- function(f) {
    memoise(f, cache = cd)
}

load_names_raw <- memoised(function() {
    names_raw <- load_csv("names.csv") %>%
        rename(instance_address = instance, instance = name) %>%
        select(instance, instance_address, folder) %>%
        distinct()

    print(colnames(names_raw))

    return(names_raw)
})

load_raw_latency <- memoised(function() {
    registerDoParallel(cl = parallel_loading_datasets_small, cores = parallel_loading_datasets_small)
    raw.latency <- bind_rows(foreach(ark = METRICS_ARKS) %dopar% {
        load_single_csv(ark, "neighbor_latency.csv") %>%
            prepare() %>%
            prepare_convert() %>%
            inner_join(load_names_raw() %>% rename(instance_to = instance_address, destination_name = instance), c("instance_to", "folder")) %>%
            mutate(destination_name = to_snake_case(destination_name))
    })

    gc()

    colnames(raw.latency)
    return(raw.latency)
})

load_node_connections <- memoised(function() {
    node_connections <- load_csv("network_shape.csv") %>%
        mutate(latency = as.numeric(latency)) %>%
        mutate(source = to_snake_case(source), destination = to_snake_case(destination))

    return(node_connections)
})

load_node_levels <- memoised(function() {
    node_levels <- load_csv("node_levels.csv") %>%
        rename(name = source, level_value = level) %>%
        mutate(
            name = to_snake_case(name),
            level = case_when(
                level_value == 0 ~ "Cloud",
                level_value == max(level_value) - 1 ~ "Edge+1",
                level_value == max(level_value) ~ "Edge",
                TRUE ~ paste("Cloud", as.character(level_value), sep = "+")
            )
        )

    return(node_levels)
})

load_latency <- memoised(function(node_connections) {
    node_connections_renamed <- node_connections %>%
        rename(instance = source, destination_name = destination, goal = latency)

    latency <- load_raw_latency() %>%
        select(destination_name, field, value, instance, timestamp, folder, metric_group, metric_group_group) %>%
        inner_join(node_connections_renamed %>%
            full_join(node_connections_renamed %>%
                mutate(toto = instance, instance = destination_name, destination_name = toto))) %>%
        mutate(diff = value - goal)
    return(latency)
})

load_functions <- memoised(function() {
    registerDoParallel(cl = parallel_loading_datasets_small, cores = parallel_loading_datasets_small)
    functions.refused <- foreach(ark = METRICS_ARKS) %dopar% {
        tryCatch(
            {
                df <- load_single_csv(ark, "refused_function_gauge.csv") %>%
                    prepare() %>%
                    prepare_convert() %>%
                    extract_function_name_info() %>%
                    select(instance, sla_id, folder, metric_group, metric_group_group, load_type, latency_type) %>%
                    # distinct() %>%
                    mutate(status = "refused") %>%
                    group_by(instance, folder, metric_group, metric_group_group, load_type, latency_type, status) %>%
                    summarise(n = n())
                return(df)
            },
            error = function(cond) {
                df <- data.frame(instance = character(0), folder = character(0), metric_group = character(0), metric_group_group = character(0), load_type = character(0), latency_type = character(0), status = character(0), n = numeric(0))
                return(df)
            }
        )
    }
    functions.refused <- bind_rows(functions.refused)

    registerDoParallel(cl = parallel_loading_datasets_small, cores = parallel_loading_datasets_small)
    functions.failed <- foreach(ark = METRICS_ARKS) %dopar% {
        tryCatch(
            {
                df <- load_single_csv(ark, "send_fails.csv") %>%
                    prepare() %>%
                    prepare_convert() %>%
                    rename(function_name = tag) %>%
                    extract_function_name_info() %>%
                    select(instance, folder, metric_group, metric_group_group) %>%
                    # distinct() %>%
                    mutate(status = "failed") %>%
                    group_by(instance, folder, metric_group, metric_group_group, status) %>%
                    summarise(n = n())
                return(df)
            },
            error = function(cond) {
                df <- data.frame(instance = character(0), folder = character(0), metric_group = character(0), metric_group_group = character(0), status = character(0), n = numeric(0))
                return(df)
            }
        )
    }

    functions.failed <- bind_rows(functions.failed)

    functions <- load_csv("provisioned_function_gauge.csv") %>%
        prepare() %>%
        prepare_convert() %>%
        extract_function_name_info() %>%
        select(instance, sla_id, folder, metric_group, metric_group_group) %>%
        # distinct() %>%
        mutate(status = "provisioned") %>%
        group_by(instance, folder, metric_group, metric_group_group, status) %>%
        summarise(n = n()) %>%
        full_join(functions.refused) %>%
        full_join(functions.failed) %>%
        {
            .
        }


    return(functions)
})

load_functions_total <- memoised(function(functions) {
    total <- functions %>%
        group_by(folder, instance, metric_group, metric_group_group, load_type, latency_type) %>%
        summarise(total = sum(n))

    functions_total <- functions %>%
        inner_join(total, by = c("instance", "folder", "metric_group", "metric_group_group", "load_type", "latency_type")) %>%
        # inner_join(node_levels %>% mutate(instance = name) %>% select(-name), by = c("instance")) %>%
        group_by(folder, status, metric_group, metric_group_group, load_type, latency_type) %>%
        summarise(total = sum(total), n = sum(n)) %>%
        mutate(ratio = n / total) %>%
        {
            .
        }

    return(functions_total)
})

load_bids_raw <- memoised(function() {
    bids_raw <- load_csv("bid_gauge.csv") %>%
        prepare() %>%
        prepare_convert()
    # bids_raw %>% filter(value <= 0) %>% select(folder) %>% distinct()

    bids_raw <- bids_raw %>% mutate(value = ifelse(value < 0 & value >= -0.001, 0, value))
    bids_raw %>% filter(value < 0)
    stopifnot(bids_raw %>% filter(value < 0) %>% summarise(n = n()) == 0)
    return(bids_raw)
})

load_provisioned_sla <- memoised(function() {
    provisioned_sla <- load_csv("function_deployment_duration.csv") %>%
        prepare() %>%
        prepare_convert() %>%
        select(bid_id, sla_id, folder, metric_group, metric_group_group, function_name) %>%
        distinct() %>%
        {
            .
        }
    colnames(provisioned_sla)
    # slice_sample(provisioned_sla, n=5)
    return(provisioned_sla)
})

load_bids_won_function <- memoised(function(bids_raw, provisioned_sla) {
    bids_won_function <- bids_raw %>%
        select(sla_id, bid_id, instance, function_name, folder, metric_group, metric_group_group, value) %>%
        distinct() %>%
        inner_join(provisioned_sla, by = c("bid_id", "sla_id", "folder", "metric_group", "metric_group_group", "function_name")) %>%
        mutate(winner = instance) %>%
        mutate(cost = value) %>%
        select(sla_id, function_name, folder, metric_group, metric_group_group, winner, cost) %>%
        {
            .
        }

    return(bids_won_function)
})

load_raw_cpu_observed_from_fog_node <- memoised(function() {
    registerDoParallel(parallel_loading_datasets_small)
    raw_cpu_observed_from_fog_node <- foreach(ark = METRICS_ARKS) %dopar% {
        cpu <- load_single_csv(ark, "cpu_observed_from_fog_node.csv") %>%
            prepare() %>%
            prepare_convert()
        cpu %>%
            filter(field == "initial_allocatable") %>%
            rename(initial_allocatable = value) %>%
            inner_join(cpu %>%
                filter(field == "used") %>%
                rename(used = value), by = c("timestamp", "folder", "instance", "metric_group", "metric_group_group")) %>%
            mutate(usage = used / initial_allocatable) %>%
            select(instance, timestamp, usage, folder, metric_group, metric_group_group)
    }
    raw_cpu_observed_from_fog_node <- bind_rows(raw_cpu_observed_from_fog_node)
    return(raw_cpu_observed_from_fog_node)
})

load_auc_usage_cpu <- memoised(function() {
    registerDoParallel(cl = parallel_loading_datasets_small, cores = parallel_loading_datasets_small)
    raw_auc_usage_cpu <- bind_rows(foreach(ark = METRICS_ARKS) %dopar% {
        load_single_csv(ark, "cpu_observed_from_fog_node.csv") %>%
            prepare() %>%
            prepare_convert() %>%
            get_usage()
    })

    gc()
    return(raw_auc_usage_cpu)
})

load_auc_usage_mem <- memoised(function() {
    raw_auc_usage_mem <- bind_rows(foreach(ark = METRICS_ARKS) %dopar% {
        load_single_csv(ark, "memory_observed_from_fog_node.csv") %>%
            prepare() %>%
            prepare_convert() %>%
            get_usage()
    })

    gc()
    return(raw_auc_usage_mem)
})

load_total_gains <- memoised(function(bids_won_function) {
    total_gains <- bids_won_function %>%
        group_by(folder, metric_group, metric_group_group, winner) %>%
        summarise(earnings = sum(cost)) %>%
        {
            .
        }
    return(total_gains)
})

load_grand_total_gains <- memoised(function(bids_won_function) {
    grand_total_gains <- bids_won_function %>%
        group_by(folder, metric_group, metric_group_group) %>%
        summarise(grand_total = sum(cost))
    return(grand_total_gains)
})

load_errors <- memoised(function() {
    errors <- tryCatch(
        {
            load_csv("iot_emulation_http_request_to_processing_echo_fails.csv") %>%
                prepare() %>%
                prepare_convert() %>%
                extract_function_name_info() %>%
                distinct()
        },
        error = function(cond) {
            columns <- c("instance", "job", "timestamp", "tag", "period", "folder", "metric_group", "latency", "value")
            df <- data.frame(
                instance = character(0),
                job = character(0),
                period = numeric(0),
                folder = character(0),
                metric_group = character(0),
                latency = character(0),
                value = numeric(0)
            )
            return(df)
        }
    )
    return(errors)
})

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

load_raw_deployment_times <- memoised(function() {
    raw.deployment_times <- load_csv("function_deployment_duration.csv") %>%
        prepare() %>%
        prepare_convert() %>%
        extract_function_name_info()
    colnames(raw.deployment_times)
    head(raw.deployment_times %>% select(function_name, everything()))
    return(raw.deployment_times)
})

load_earnings_jains_plot_data <- memoised(function(node_levels, bids_won_function) {
    earnings.jains.plot.data.raw <- node_levels %>%
        rename(winner = name) %>%
        full_join(bids_won_function %>% group_by(folder, winner, metric_group) %>% summarise(earnings = sum(cost))) %>%
        mutate(earnings = ifelse(is.na(earnings), 0, earnings)) %>%
        group_by(metric_group, folder) %>%
        summarise(jains_index = jains_index(earnings), worst_case = round(1 / n(), 2), n = n()) %>%
        rename(score = jains_index) %>%
        # left_join(raw.nb_functions) %>%
        # left_join(raw.nb_functions.total.full %>% rename(total_func = total)) %>%
        # left_join(raw.nb_functions.total.ll %>% rename(total_func_ll = total)) %>%
        # rowwise() %>%
        # mutate(ratio_func_ll = total_func_ll / total_func) %>%
        correct_names()

    return(earnings.jains.plot.data.raw)
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

load_respected_sla <- function() {
    registerDoParallel(cl = parallel_loading_datasets, cores = parallel_loading_datasets)
    respected_sla.header.nb <- foreach(ark = METRICS_ARKS) %dopar% {
        gc()

        load_single_csv(ark, "proxy.csv") %>%
            prepare() %>%
            group_by(folder) %>%
            adjust_timestamps(var_name = "timestamp", reference = "value_raw") %>%
            adjust_timestamps(var_name = "value_raw", reference = "value_raw") %>%
            mutate(value = value_raw) %>%
            rename(function_name = tags) %>%
            group_by(folder, metric_group, metric_group_group, req_id) %>%
            arrange(value) %>%
            mutate(ran_for = timestamp - value) %>%
            mutate(prev = ifelse(first_req_id == TRUE, value, timestamp)) %>%
            mutate(in_flight = value - lag(prev)) %>%
            mutate(total_process_time = first(ran_for)) %>%
            filter(first_req_id == FALSE) %>%
            extract_function_name_info() %>%
            extract_functions_pipeline() %>%
            group_by(sla_id, folder, metric_group, metric_group_group, function_name, docker_fn_name, pipeline, total_process_time) %>%
            summarise(
                satisfied_count = sum(in_flight <= latency),
                acceptable_count = sum(in_flight <= latency + 0.001),
                total = n(),
                measured_latency = mean(in_flight),
                ran_for = max(ran_for),
                errored = sum(status != 200)
            ) %>%
            mutate(count.satisfied = satisfied_count / total) %>%
            mutate(count.acceptable = acceptable_count / total) %>%
            {
                .
            }
    }

    respected_sla.header.nb <- bind_rows(respected_sla.header.nb)

    gc()
    return(respected_sla.header.nb)
}

load_nb_deployed_plot_data <- memoised(function(respected_sla.header.nb, functions_total, node_levels) {
    plots.nb_deployed.data <- respected_sla.header.nb %>%
        extract_function_name_info() %>%
        left_join(functions_total %>% select(folder, metric_group, metric_group_group, load_type, latency_type, total) %>% rename(nb_functions_requested_total = total)) %>%
        left_join(node_levels %>% group_by(folder) %>% summarise(nb_nodes = n())) %>%
        # mutate(nb_nodes_group = case_when(
        #     nb_nodes < 19 ~ "Danger zone 1",
        #     nb_nodes <= 34 ~ "$19 \\le n < 34$",
        #     nb_nodes < 112 ~ "Danger zone 2",
        #     nb_nodes <= 119 ~ "$112 \\le n \\le 119$",
        #     TRUE ~ "Danger zone 3",
        # )) %>%
        rename(nb_nodes_group = nb_nodes) %>%
        group_by(folder, metric_group, nb_nodes_group, nb_functions_requested_total) %>%
        summarise(funcs = n()) %>%
        mutate(nb_functions = funcs / nb_functions_requested_total) %>%
        correct_names()

    return(plots.nb_deployed.data)
})

load_respected_sla_plot_data <- memoised(function(respected_sla.header.nb) {
    plots.respected_sla.data <- respected_sla.header.nb %>%
        extract_function_name_info() %>%
        # left_join(raw.nb_functions.total.full %>% rename(total_func = total)) %>%
        # left_join(raw.nb_functions.total.ll %>% rename(total_func_ll = total)) %>%
        rowwise() %>%
        # mutate(ratio_func_ll = total_func_ll / total_func) %>%
        mutate(satisfied_count = count.acceptable) %>%
        # mutate(satisfied_count = abs(measured_latency - latency) / latency) %>%
        mutate(measured_latency = abs(measured_latency) / latency) %>%
        # mutate(ratio_func_ll = sprintf("%.1f%% low-latency ƒ", ratio_func_ll *100))  %>%
        correct_names() %>%
        mutate(toto = "toto")
    # mutate(group = sprintf("%s\n%s\n%s", latency_type, load_type, ratio_func_ll))
    # mutate(group = latency_type)
    return(plots.respected_sla.data)
})

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
        mutate(y = errored) %>%
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

output_in_flight_time_plot_simple <- function(respected_sla, bids_won_function, node_levels) {
    df <- respected_sla %>%
        mutate(measured_latency = as.numeric(measured_latency)) %>%
        select(-sla_id) %>%
        left_join(bids_won_function %>% ungroup() %>% select(folder, winner, sla_id) %>% distinct()) %>%
        left_join(node_levels %>% rename(winner = name)) %>%
        {
            .
        }
    p <- ggplot(data = df, aes(x = level_value, y = measured_latency, color = docker_fn_name, alpha = 1)) +
        facet_grid(rows = vars(pipeline)) +
        scale_color_viridis(discrete = TRUE) +
        scale_fill_viridis(discrete = TRUE) +
        scale_y_continuous(trans = "log10") +
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

output_ran_for_plot_simple <- function(respected_sla) {
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

load_spending_plot_data <- memoised(function(bids_won_function) {
    plots.spending.data <- bids_won_function %>%
        extract_function_name_info() %>%
        # left_join(raw.nb_functions) %>%
        # mutate(ratio_func_ll = total_func_ll / total_func) %>%
        # mutate(ratio_func_ll = sprintf("%.1f%% ll ƒ", ratio_func_ll *100)) %>%
        group_by(folder, metric_group) %>%
        summarise(spending = mean(cost)) %>%
        # mutate(fn_category = sprintf("%s\n%s", latency_type, load_type)) %>%
        correct_names()
    return(plots.spending.data)
})

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

    p <- ggplot(data = df, aes(x = docker_fn_name, y = cost, color = folder, alpha = 1)) +
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
        guides(colour = guide_legend(nrow = 1)) +
        geom_beeswarm()

    return(p)
}

respected_sla <- load_respected_sla()
node_levels <- load_node_levels()
bids_raw <- load_bids_raw()
provisioned_sla <- load_provisioned_sla()
bids_won_function <- load_bids_won_function(bids_raw, provisioned_sla)

ggsave("respected_sla_simple.png", output_respected_data_plot_simple(respected_sla, bids_won_function, node_levels))
ggsave("errored.png", output_errored_plot_simple(respected_sla, bids_won_function, node_levels))
ggsave("in_flight_time.png", output_in_flight_time_plot_simple(respected_sla, bids_won_function, node_levels))
ggsave("ran_for.png", output_ran_for_plot_simple(respected_sla))

stop()

node_connections <- load_node_connections()
latency <- load_latency(node_connections)
output_latency(latency)
ggsave("output_latency.png")

raw.cpu.observed_from_fog_node <- load_raw_cpu_observed_from_fog_node()
if (generate_gif) {
    output_gif(raw.cpu.observed_from_fog_node, bids_won_function)
}


earnings_jains_plot_data <- load_earnings_jains_plot_data(node_levels, bids_won_function)
# ggsave("jains.png", output_jains(earnings_jains_plot_data))



functions <- load_functions()
functions_total <- load_functions_total(functions)


# plots.nb_deployed.data <- load_nb_deployed_plot_data(respected_sla, functions_total, node_levels)
# # ggsave("anova_nb_deployed.png", output_anova_nb_deployed(plots.nb_deployed.data))

# plots.respected_sla <- load_respected_sla_plot_data(respected_sla)
# # ggsave("respected_sla.png", output_respected_data_plot(plots.respected_sla))

# ggsave("jains.png", output_jains_index_plot(earnings_jains_plot_data))
raw_deployment_times <- load_raw_deployment_times()
# ggsave("mean_time_to_deploy.png", output_mean_time_to_deploy(raw_deployment_times))
ggsave("mean_time_to_deploy_simple.png", output_mean_time_to_deploy_simple(raw_deployment_times))

# spending_plot_data <- load_spending_plot_data(bids_won_function)
# ggsave("spending.png", output_spending_plot(spending_plot_data))
ggsave("spending_simple.png", output_spending_plot_simple(bids_won_function))
# options(width = 1000)
# toto <- load_csv("proxy.csv") %>%
#     # rename(function_name = tags) %>%
#     # extract_function_name_info() %>%
#     filter(req_id == "063ea3fa-b428-4977-a1e4-7588c326b8a4") %>%
#     {
#         .
#     }

# print(toto)
