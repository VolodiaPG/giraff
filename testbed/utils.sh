echodate(){
    echo `date +%Y-%m-%d_%H:%M:%S`
}

log(){
  text=$1
  skip=$2
  # rest of args are the command

  if [ "$skip" = "true" ]; then
    echo -e "$text \033[34mSKIPPING\033[0m"
    return
  fi

  shift # Remove arg
  shift # Remove arg

  cmd=( "$@" )

  file=$(mktemp)
  status_code_file=$(mktemp)


  do_command() {
    ${cmd[@]} > $file 2>&1
    local status=$?
    echo $status > $status_code_file
  }
  do_command &
  cmd_pid=$!
  # Wait for seconds and check if the command is still running
  do_starting_block() {
      timestamp=$(echodate)
      sleep 3
      if kill -0 $cmd_pid 2>/dev/null; then
        echo -e "\033[37m$timestamp\033[0m $text \033[33m...starting\033[0m"
          wait $cmd_pid 2>/dev/null # Optionally wait for the command to actually finish
      else
          wait $cmd_pid 2>/dev/null # Command finished within 2 seconds
      fi
  }

  do_starting_block &
  wait $cmd_pid
  status=$(cat $status_code_file)
  if [ $status -eq 0 ]; then
    echo -e "\033[37m$(echodate)\033[0m $text \033[32mOK\033[0m"
    rm $file
  else
    echo -e "\033[37m$(echodate)\033[0m $text \033[31mFAILED\033[0m"
    echo -en "\033[31mErr is: \033[0m"
    cat $file
    rm $file
  fi
  return $status
}

