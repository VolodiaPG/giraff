#!/usr/bin/env bash

MAX_LATENCY=$1
TARGET_NODE=$2
NB_FUNCTIONS=$3
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
configs_mem="150"

#configs_cpu=("100" "150" "500") #millicpu
configs_cpu="100"

size=${#configs_cpu[@]}

iot_requests_body=()

lat_index=0

ii=0
jj=0
until [ $ii -ge $NB_FUNCTIONS ]; do
	function_id=$(printf "%03d" $jj)

	latency=$(($RANDOM % ($MAX_LATENCY - 5) + 5))

	mem="$configs_mem"
	cpu="$configs_cpu"
	docker_fn_name='echo'
	function_name="$docker_fn_name--$function_id--$latency--$cpu--$mem"

	echo -e "${ORANGE}Doing function ${function_name}${DGRAY}" # DGRAY for the following

	response=$(curl --silent --output response.tmp --write-out "%{http_code}" --request PUT \
		--url "http://localhost:$PORT/api/function" \
		--header 'Content-Type: application/json' \
		--data '{
	"sla": {
		"storage": "0 MB",
		"memory": "'"$mem"' MB",
		"cpu": "'"$cpu"' millicpu",
		"latencyMax": "'"$latency"' ms",
		"maxReplica": 1,
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
	if [ $response == 200 ]; then
		FUNCTION_ID=$(cat response.tmp)
		rm response.tmp
		FAAS_IP=$(echo "$FUNCTION_ID" | jq -r .chosen.ip)
		FAAS_PORT=$(echo "$FUNCTION_ID" | jq -r .chosen.port)
		FUNCTION_ID=$(echo "$FUNCTION_ID" | jq -r .chosen.bid.id)
		echo -e "${GREEN}${FUNCTION_ID}${DGRAY}" # DGRAY for the following

		iot_requests_body+=('{
		"iotUrl": "http://'$IOT_URL':'$IOT_LOCAL_PORT'/api/print",
		"nodeUrl": "http://'$FAAS_IP':'$FAAS_PORT'/function/'$function_name'-'$FUNCTION_ID'",
		"functionId": "'$FUNCTION_ID'",
		"tag": "'"$function_name"'"
		}')
	else
		echo -e "${RED}$(cat response.tmp)${NC}"
		ii=$((--ii))
		sleep 2
	fi

	ii=$((++ii))
	jj=$((++jj))
done

echo -e "${PURPLE}Instanciating echoes from Iot platform for all the functions instanciated ${RED}" # RED for the following

for body in "${iot_requests_body[@]}"; do
	curl --request PUT \
		--url http://localhost:$IOT_LOCAL_PORT/api/cron \
		--header 'Content-Type: application/json' \
		--data "$body"
	echo -e "\n${GREEN}Iot registred${RED}" # DGRAY for the following
done
