# output_spending_plot <- function(plots.spending.data) {
#   df <- plots.spending.data %>%
#     mutate(group = "toto") %>%
#     ungroup()
#
#   p <- ggplot(data = df, aes(alpha = 1)) +
#     #  facet_grid(~var_facet) +
#     theme(legend.position = "none") +
#     scale_alpha_continuous(guide = "none") +
#     labs(
#       y = "Function cost",
#       x = "Placement method",
#     ) +
#     theme(
#       legend.background = element_rect(
#         fill = alpha("white", .7),
#         size = 0.2,
#         color = alpha("white", .7)
#       )
#     ) +
#     theme(
#       legend.spacing.y = unit(0, "cm"),
#       legend.margin = margin(0, 0, 0, 0),
#       legend.box.margin = margin(-10, -10, -10, -10),
#       axis.text.x = element_text(angle = 15, vjust = 1, hjust = 1)
#     ) +
#     # scale_x_discrete(guide = guide_axis(n.dodge = 2)) +
#     # theme(legend.position = c(.5, .93)) +
#     scale_color_viridis(discrete = T) +
#     scale_fill_viridis(discrete = T) +
#     guides(colour = guide_legend(nrow = 1))
#
#   mean_cb <- function(Letters, mean) {
#     {
#       return(sprintf("%s\n\\footnotesize{$\\mu=%.1f$}", Letters, mean))
#     } %>M%
#       plots.spending <- anova_boxplot(
#       p,
#       df,
#       "Placement method",
#       "spending",
#       "group",
#       mean_cb
#     )
#   }
#   plots.spending + labs(title = plots.spending.caption)
#   return(plots.spending)
# }
