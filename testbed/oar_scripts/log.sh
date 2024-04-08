#!/usr/bin/env bash
for oarjob in `oarstat -u -f -J | jq -r ".[] | @base64"` ; do
	name=`echo $oarjob | base64 --decode | jq -r .name`
    start_time=`echo $oarjob | base64 --decode | jq -r .start_time`
    start_time=`date -d @"$start_time" +%Y%m%d`
    echo "$HOME/logs_campaign/$name-$start_time.log"
done

