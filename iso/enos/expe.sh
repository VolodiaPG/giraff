#!/usr/bin/env bash

MAX=$1
TARGET_NODE=$2
DELAY=$3
PORT=$4
IOT_LOCAL_PORT=$5
IOT_URL=$6
TARGET_REMOTE_IP=$7

# Colors
RED='\033[0;31m'
ORANGE='\033[0;33m'
PURPLE='\033[0;34m'
DGRAY='\033[0;30m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

#configs_mem=("50" "150" "500") # megabytes
configs_mem=("100" "100" "100" "100" "100")

configs_latency=("50" "100" "300" "500" "1000") # ms

#configs_cpu=("100" "150" "500") #millicpu
configs_cpu=("200" "200" "200" "200" "200")

size=${#configs_cpu[@]}

for ii in $(seq 1 $MAX)
do
	function_id=$(printf "%03d" $ii)


	index=$(($ii % $size))
	mem="${configs_mem[$index]}"
	cpu="${configs_cpu[$index]}"
	latency="${configs_latency[$index]}"
	docker_fn_name='echo'
	function_name="$docker_fn_name-$function_id-$latency-$cpu-$mem"
	
	echo -e "${ORANGE}Doing function ${function_name}${DGRAY}" # DGRAY for the following

	FUNCTION_ID=$(curl --request PUT \
  --url "http://localhost:$PORT/api/function" \
  --header 'Content-Type: application/json' \
  --data '{
	"sla": {
		"storage": "0 MB",
		"memory": "'"$mem"' MB",
		"cpu": "'"$cpu"' millicpu",
		"latencyMax": "'"$latency"' ms",
		"dataInputMaxSize": "1 GB",
		"dataOutputMaxSize": "1 GB",
		"maxTimeBeforeHot": "10 s",
		"reevaluationPeriod": "1 hour",
		"functionImage": "ghcr.io/volodiapg/'"$docker_fn_name"':latest",
		"functionLiveName": "'"$function_name"'",
		"dataFlow": [
			{
				"from": {
					"dataSource": "'"$TARGET_NODE"'"
				},
				"to": "thisFunction"
			}
		]
	},
	"targetNode": "'"$TARGET_NODE"'"
  }')
	echo -e $FUNCTION_ID
	FUNCTION_ID=$(echo "$FUNCTION_ID" | jq -r .chosen.bid.id)
	echo -e "${GREEN}${FUNCTION_ID}${NC}" # DGRAY for the following

	sleep $DELAY
	echo -e "${PURPLE}Instanciating echo from Iot platform${RED}" # DGRAY for the following


  	curl --request PUT \
  --url http://localhost:$IOT_LOCAL_PORT/api/cron \
  --header 'Content-Type: application/json' \
  --data '{
	"iotUrl": "http://'$IOT_URL':3030/api/print",
	"firstNodeUrl": "http://'$TARGET_REMOTE_IP':3030/api/routing",
	"functionId": "'$FUNCTION_ID'",
	"tag": "'"$function_name"'"
  }'
echo -e "\n${GREEN}Iot registred${NC}" # DGRAY for the following
sleep $DELAY

done
