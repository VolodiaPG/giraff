#!/bin/bash
echo "To enable tmux mouse interaction, add 'set -g mouse' on to your .tmux.conf"
for ID in $(seq 1 "${nb_instances}"); do
  echo "Starting instance $ID"
  if [[ $ID -eq 1 ]]; then
      tmux new-session -d -s my_session_name "printf '\033]2;%s\033\\' '${ID} — $(cat node-situation-${ID}.ron | sed -n 's/\s*my_id\s*:\s*\"\(.*\)\"/ \1 /p')' ; source start_node.sh ${ID} ; echo 'Waiting for keypress to close...' && read" #-n "toto" #"#${ID} — $(cat node-situation-${ID}.ron | sed -n 's/\s*my_id\s*:\s*\"\(.*\)\"/ \1 /p')"
  else
      tmux split-window -d -t my_session_name:0 -p20 -v "printf '\033]2;%s\033\\' '${ID} — $(cat node-situation-${ID}.ron | sed -n 's/\s*my_id\s*:\s*\"\(.*\)\"/ \1 /p')' ; source start_node.sh ${ID}; echo 'Waiting for keypress to close...' && read" #-n "totot" #"#${ID} — $(cat node-situation-${ID}.ron | sed -n 's/\s*my_id\s*:\s*\"\(.*\)\"/ \1 /p')"
  fi
  sleep 1
done
tmux select-layout -t my_session_name:0 tiled
tmux attach-session -t my_session_name


