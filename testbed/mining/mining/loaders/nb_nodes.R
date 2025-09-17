load_nb_nodes <- function(node_levels) {
  node_levels %>%
    group_by(folder, metric_group) %>%
    summarise(nb_nodes = n())
}
