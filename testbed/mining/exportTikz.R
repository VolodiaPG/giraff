if (length(find.package("tikzDevice", quiet = TRUE))) {
    library(tikzDevice)
    options(tikzDefaultEngine = "xetex")
    options(
        tikzSanitizeCharacters = c("%", "&", "#", "\\$", "\\{", "\\}", "\\^", "\\~", "ƒ"),
        tikzReplacementCharacters = c("\\%", "\\&", "\\#", "\\\\$", "\\\\{", "\\\\}", "\\\\^", "\\\\~", "$f$")
    )
    options(tikzXelatexPackages = c(
        "\\usepackage{tikz}\n",
        "\\usepackage[active,tightpage,xetex]{preview}\n",
        "\\usepackage{fontspec,xunicode}\n",
        "\\PreviewEnvironment{pgfpicture}\n",
        "\\setlength\\PreviewBorder{0pt}\n",
        "\\newcommand{\\dash}{-}\n"
    ))
    options(
        tikzMetricPackages = c(
            "\\usepackage[T1]{fontenc}\n",
            "\\usetikzlibrary{calc}\n"
        ),
        tikzUnicodeMetricPackages = c(
            "\\usepackage[T1]{fontenc}\n",
            "\\usetikzlibrary{calc}\n",
            "\\usepackage{fontspec,xunicode}\n"
        )
    )
    # options(tikzMetricPackages = c("\\usepackage{preview}", "\\usepackage{pgf}", "\\usepackage{xcolor}"))

    tikzDevice::tikzTest()

    column_width <- 7

    export_graph <- function(name, plot, width, height, caption) {
        tikzDevice::tikz(name, width = width, height = height, standAlone = FALSE, sanitize = TRUE) # , standAlone = TRUE)
        print(plot)
        dev.off()

        # read the existing content of the file
        existing_content <- readLines(name)

        tex_width <- 1
        if (width > GRAPH_ONE_COLUMN_WIDTH) {
            tex_width <- 2
        }

        new_content <- c(
            sprintf("\\newcommand{\\dash}{-}\n\\resizebox{%s\\columnwidth}{!}{", tex_width),
            existing_content,
            "}",
            sprintf("\\caption{%s}", caption)
        )

        # write the new content to the file
        writeLines(new_content, name)

        fig(width, height)
        print(plot + labs(title = caption))
    }


    # tikzDevice::tikz("./nodes.tex", width = 15, height = 5, standAlone = FALSE, sanitize = TRUE) # , standAlone = TRUE)
    # plot(net_connected, layout = coords, asp = 0.22, margin = -0, edge.label = E(net_connected)$weight, edge.width = 1, vertex.size = 5, vertex.label.cex = 1, vertex.dist = 20, edge.arrow.size = 0.5, edge.label.cex = 0.8, edge.label.dist = 1.5)
    # dev.off()

    export_graph("./jain.tex", plots.jains, plots.jains.w, plots.jains.h, plots.jains.caption)
    export_graph("./nb_deployed.tex", plots.nb_deployed, plots.nb_deployed.w, plots.nb_deployed.h, plots.nb_deployed.caption)
    export_graph("./respected_sla.tex", plots.respected_sla, plots.respected_sla.w, plots.respected_sla.h, plots.respected_sla.caption)
    # export_graph("./skew.tex", plots.skew, plots.skew.w, plots.skew.h, plots.skew.caption)
    export_graph("./spending.tex", plots.spending, plots.spending.w, plots.spending.h, plots.spending.caption)
    export_graph("./deploymenttimes.tex", plots.deploymenttimes, plots.deploymenttimes.w, plots.deploymenttimes.h, plots.deploymenttimes.caption)

    # output_file <- "nb_experiences_per_categories.tex"
    # tibble_to_latex_tabular(nb_experiences_per_categories, output_file)
}
