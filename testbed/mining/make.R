library(targets)
library(tools)

# Function to monitor R files and rebuild when changes are detected
monitor_and_build <- function() {
  # Get all R files in current directory and subdirectories
  r_files <- list.files(pattern = "\\.R$", recursive = TRUE, full.names = TRUE)
  
  # Get the latest modification time of any R file
  get_latest_mtime <- function() {
    max(file.mtime(r_files))
  }
  
  # Store the initial modification time
  last_build_time <- Sys.time()
  last_file_mtime <- get_latest_mtime()
  
  cat("Monitoring R files for changes...\n")
  
  # Infinite loop to monitor files
  repeat {
    Sys.sleep(1)  # Check every second
    
    # Get the latest modification time
    current_file_mtime <- tryCatch({
      get_latest_mtime()
    }, error = function(e) {
      # If files were deleted, continue monitoring
      return(last_file_mtime)
    })
    
    # If any R file was modified since last build, rebuild
    if (current_file_mtime > last_build_time) {
      cat("Detected changes in R files. Rebuilding...\n")
      targets::tar_make()
      last_build_time <- Sys.time()
    }
  }
}

# Start monitoring
monitor_and_build()
