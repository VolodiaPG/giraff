flame_functions <- function(spans) {
  spans %>%
    filter(startsWith(span.name, "FLAME") & endsWith(span.name, "...")) %>%
    mutate(span.name = substring(span.name, 1, nchar(span.name) - 3))
}
