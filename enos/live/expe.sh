#!/usr/bin/env bash
TARGET_NODE=$1
IOT_IP=$2

MARKET_LOCAL_PORT="${MARKET_LOCAL_PORT:=8088}"
IOT_LOCAL_PORT="${IOT_LOCAL_PORT:=3003}"

MAX_LATENCY_LOW_LATENCY="${MAX_LATENCY_LOW_LATENCY:=10}"
MIN_LATENCY_LOW_LATENCY="${MIN_LATENCY_LOW_LATENCY:=5}"
MAX_LATENCY_REST_LATENCY="${MAX_LATENCY_REST_LATENCY:=75}"
MIN_LATENCY_REST_LATENCY="${MIN_LATENCY_REST_LATENCY:=45}"
NB_FUNCTIONS_LOW_LATENCY="${NB_FUNCTIONS_LOW_LATENCY:=50}"
NB_FUNCTIONS_REST="${NB_FUNCTIONS_REST:=50}"

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

function_latencies=()

for ii in $(seq 1 $NB_FUNCTIONS_LOW_LATENCY); do
	function_latencies+=(
		$(($RANDOM % ($MAX_LATENCY_LOW_LATENCY - $MIN_LATENCY_LOW_LATENCY) + $MIN_LATENCY_LOW_LATENCY))
	)
done
for ii in $(seq 1 $NB_FUNCTIONS_REST); do
	function_latencies+=(
		$(($RANDOM % ($MAX_LATENCY_REST_LATENCY - $MIN_LATENCY_REST_LATENCY) + $MIN_LATENCY_REST_LATENCY))
	)
done

# Shuffle
function_latencies=($(shuf -e "${function_latencies[@]}"))

ii=0
for latency in "${function_latencies[@]}"; do
	function_id=$(printf "%03d" $ii)

	mem="$configs_mem"
	cpu="$configs_cpu"
	docker_fn_name='echo'
	function_name="$docker_fn_name--$function_id--$latency--$cpu--$mem"

	echo -e "${ORANGE}Doing function ${function_name}${DGRAY}" # DGRAY for the following

	response=$(curl --silent --output response.tmp --write-out "%{http_code}" --request PUT \
		--url "http://localhost:$MARKET_LOCAL_PORT/api/function" \
		--header 'Content-Type: application/json' \
		--data '{
	"sla": {
		"memory": "'"$mem"' MB",
		"cpu": "'"$cpu"' millicpu",
		"latencyMax": "'"$latency"' ms",
		"maxReplica": 1,
		"reservationEndAt": 1677661778,
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
		"iotUrl": "http://'$IOT_IP':'$IOT_LOCAL_PORT'/api/print",
		"nodeUrl": "http://'$FAAS_IP':'$FAAS_PORT'/function/'$function_name'-'$FUNCTION_ID'",
		"functionId": "'$FUNCTION_ID'",
		"tag": "'"$function_name"'"
		}')
	else
		echo -e "${RED}$(cat response.tmp)${NC}"
	fi

	ii=$((++ii))
done

echo -e "${PURPLE}Instanciating echoes from Iot platform for all the functions instanciated ${RED}" # RED for the following

for body in "${iot_requests_body[@]}"; do
	curl --request PUT \
		--url http://localhost:$IOT_LOCAL_PORT/api/cron \
		--header 'Content-Type: application/json' \
		--data "$body"
	echo -e "\n${GREEN}Iot registred${RED}" # DGRAY for the following
done
