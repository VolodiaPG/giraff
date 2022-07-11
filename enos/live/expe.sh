#! /bin/bash

MAX=$1
TARGET_NODE=$2
DELAY=$3
PORT=$4

for ii in $(seq 10 $END)
do
	echo $ii
	curl --request PUT \
	  --url http://localhost:$PORT/api/function \
	  --header 'Content-Type: application/json' \
	  --data '{
		"sla": {
			"storage": "0 MB",
			"memory": "100 MB",
			"cpu": "100 millicpu",
			"latencyMax": "700 ms",
			"dataInputMaxSize": "1 GB",
			"dataOutputMaxSize": "1 GB",
			"maxTimeBeforeHot": "10 s",
			"reevaluationPeriod": "1 hour",
			"functionImage": "ghcr.io/volodiapg/primes:latest",
			"functionLiveName": "primes-'$ii'"
		},
		"targetNode": "'$TARGET_NODE'",
		"requestDestinations": [],
		"requestSources": []
	}'	
	sleep $DELAY
done
