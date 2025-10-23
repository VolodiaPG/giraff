text_profit_output <- function(otel_profit) {
  Log(colnames(otel_profit))
  profitables <- otel_profit %>%
    extract_context() %>%
    mutate(benefit = profit > 0) %>%
    group_by(env_live, env, folder) %>%
    summarise(nb_benefit = sum(benefit), nb = n()) %>%
    mutate(ratio_benefit = nb_benefit / nb) %>%
    group_by(env, env_live) %>%
    summarize(
      mean = mean(ratio_benefit),
      n = n(),
      std_dev = sd(ratio_benefit),
      margin_error = qt(0.975, df = n - 1) * std_dev / sqrt(n),
      lower_ci = mean - margin_error,
      upper_ci = mean + margin_error
    ) %>%
    mutate(file_name = paste0("profitables_", env, "_", env_live, ".txt"))

  profitables %>%
    rowwise() %>%
    group_walk(
      ~ write(
        paste0(
          round(.x$mean * 100, 1),
          "\\% \\pm ",
          round(.x$margin_error * 100, 1),
          "\\%"
        ),
        file = paste0("out/", .x$file_name)
      )
    )

  roi_increase <- otel_profit %>%
    extract_context() %>%
    group_by(env_live, folder) %>%
    summarise(roi = mean(roi)) %>%
    group_by(env_live) %>%
    summarise(roi = mean(roi)) %>%
    arrange(env_live)

  increase <- (roi_increase[2, ]$roi + roi_increase[3, ]$roi) /
    (2 * roi_increase[1, ]$roi)
  write(
    paste0(round(increase * 100, 1), "\\%"),
    file = "out/roi_increase.txt"
  )

  increase <- (roi_increase[3, ]$roi + roi_increase[4, ]$roi) /
    (roi_increase[1, ]$roi + roi_increase[2, ]$roi)
  write(
    paste0(round(increase * 100, 1), "\\%"),
    file = "out/roi_increase_2.txt"
  )
}
