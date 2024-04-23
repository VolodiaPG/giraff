log(){
  set +exm
  text=$1
  skip=$2
  # rest of args are the command

  if [ "$skip" = "true" ]; then
    echo -e "$text \033[34mSKIPPING\033[0m"
    return
  fi

  shift # Remove arg
  shift # Remove arg

  file=$(mktemp)

  $@ > $file 2>&1&
  cmd_pid=$!
  # Wait for seconds and check if the command is still running
  {
      sleep 4
      if kill -0 $cmd_pid 2>/dev/null; then
          echo -e "$text \033[33m...starting\033[0m"
          wait $cmd_pid 2>/dev/null # Optionally wait for the command to actually finish
      else
          wait $cmd_pid 2>/dev/null # Command finished within 2 seconds
      fi
  }&
  wait $cmd_pid
  status=$?
  if [ $status -eq 0 ]; then
    echo -e "$text \033[32mOK\033[0m"
    rm $file
  else
    echo -e "$text \033[31mFAILED\033[0m"
    echo -n "Err is: "
    cat $file
    rm $file
    exit $status
  fi
}

