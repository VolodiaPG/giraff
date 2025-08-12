source("config.R")

init <- function() {
  Sys.setenv(VROOM_TEMP_PATH = "./vroom")
  system("mkdir -p ./vroom")
  system("rm ./vroom/* || true")

  # To call python from R
  library(archive)
  library(dplyr)
  loadNamespace("dlookr")
  library(tidyverse)
  library(igraph)
  library(formattable)
  library(stringr)
  library(viridis)
  library(patchwork)
  library(cowplot)
  library(scales)
  library(vroom)
  library(zoo)
  library(ggprism)
  library(snakecase)
  library(foreach)
  library(doParallel)
  library(ggbeeswarm)
  library(multidplyr)
  library(multcompView)
  library(car)
  library(purrr)
  library(plotly)
  library(htmlwidgets)
  library(htmltools)
  library(memoise)
  library(future.apply)
  library(rlang)
  library(jsonlite)

  future::plan("multicore", workers = workers)

  ggplot2::theme_set(theme_prism())
}

suppressMessages(init())

Log <- function(message) {
  print(message)
}

# The rest of the logic is now handled by _targets.R
