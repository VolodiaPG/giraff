output_anova_nb_deployed <- function(plots.nb_deployed.data) {
  df <- plots.nb_deployed.data %>% ungroup()

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