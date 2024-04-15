#!/usr/bin/env bash
set +x
oarjob=$1
while true; do
  oar=`oarstat -j $oarjob -J | jq -r ".\"$oarjob\" | @base64"`
  name=`echo $oar | base64 --decode | jq -r .name`
  start_time=`echo $oar | base64 --decode | jq -r .start_time`
  start_time=`date -d @"$start_time" +%Y%m%d`
  file="./logs_campaign/$name-$start_time.log"

  if tail -f $file; then
    exit 0
  fi

  sleep 2
  echo -n "."
done
