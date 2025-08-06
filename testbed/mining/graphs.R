# Automatically source all graph plotting functions from the graphs directory

# Get the directory of this script
script_dir <- dirname(sys.frame(1)$ofile)

# List all .R files in the graphs subdirectory
graph_files <- list.files(file.path(script_dir, "graphs"), pattern = "\\.R$", full.names = TRUE)

# Source each file
for (file in graph_files) {
  source(file)
}