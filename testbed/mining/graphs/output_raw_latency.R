output_raw_latency <- function(latency) {
  fields <- latency %>%
    select(field) %>%
    distinct()

  Log(fields)

  latency %>%
    filter(field == "average") %>%
    adjust_timestamps() %>%
    rename(source = instance, destination = destination_name) %>%
    select(timestamp, source, destination, folder, value) %>%
    mutate(
      sorted_interaction = pmap_chr(
        list(source, destination),
        ~ paste(sort(c(...)), collapse = "_")
      )
    ) %>%
    ggplot(aes(
      x = sorted_interaction,
      y = value,
      color = (interaction(source, destination, sep = "_") ==
        sorted_interaction),
      group = interaction(source, destination)
    )) +
    facet_grid(cols = vars(folder)) +
    theme(axis.text.x = element_text(angle = 90, vjust = 1, hjust = 1)) +
    geom_quasirandom(method = "tukey", alpha = .2) +
    theme(legend.position = "none")
}
