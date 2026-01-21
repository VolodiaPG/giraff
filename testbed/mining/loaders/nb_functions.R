load_nb_functions <- function(spans) {
  df <- spans %>%
    extract_context() %>%
    select(
      env,
      env_live,
      folder,
      metric_group,
      service.namespace,
      service.instance.id,
      run
    ) %>%
    group_by(run, folder, metric_group, service.namespace, env, env_live) %>%
    distinct() %>%
    summarise(nb_functions = n())
}
