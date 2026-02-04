text_profit_output <- function(otel_profit) {
  Log(colnames(otel_profit))
  profitables <- otel_profit %>%
    extract_context() %>%
    mutate(benefit = profit > 0) %>%
    group_by(env_live, env, folder) %>%
    summarise(nb_benefit = sum(benefit), nb = n()) %>%
    mutate(ratio_benefit = nb_benefit / nb) %>%
    # Groups are anova letters
    mutate(group = ifelse(env_live %in% c("1", "2"), "b", "a")) %>%
    group_by(group) %>%
    summarize(
      mean = mean(ratio_benefit),
      n = n(),
      std_dev = sd(ratio_benefit),
      margin_error = qt(0.975, df = n - 1) * std_dev / sqrt(n),
      lower_ci = mean - margin_error,
      upper_ci = mean + margin_error
    ) %>%
    mutate(file_name = paste0("profitables_letter_", group, ".txt"))

  profitables %>%
    rowwise() %>%
    group_walk(
      ~ write(
        paste0(
          round(.x$mean * 100, 1),
          "\\% \\pm ",
          round(.x$std_dev * 100, 1),
          "\\%"
        ),
        file = paste0("figures/", .x$file_name)
      )
    )

  meangood <- profitables[2, ]$mean
  meanbad <- profitables[1, ]$mean
  increase <- abs(meangood - meanbad)
  write(
    paste0(round(increase * 100, 1), " percentage points"),
    file = "figures/profitable_increase.txt"
  )

  roi_increase <- otel_profit %>%
    extract_context() %>%
    group_by(env_live, folder) %>%
    summarise(roi = mean(roi)) %>%
    group_by(env_live) %>%
    summarise(roi = mean(roi)) %>%
    arrange(env_live)

  increase <- (roi_increase[3, ]$roi - roi_increase[2, ]$roi) /
    (roi_increase[2, ]$roi)
  write(
    paste0(round(increase * 100, 1), "\\%"),
    file = "figures/roi_increase.txt"
  )

  increase <- (roi_increase[4, ]$roi - roi_increase[3, ]$roi) /
    (roi_increase[3, ]$roi)
  write(
    paste0(round(increase * 100, 1), "\\%"),
    file = "figures/roi_increase_2.txt"
  )
}
