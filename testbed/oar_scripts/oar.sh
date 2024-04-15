#!/usr/bin/env bash
for oarjob in `oarstat -u -J | jq -r "to_entries | sort_by(.key | tonumber) | from_entries | .[] | @base64"` ; do
	echo -n `echo $oarjob | base64 --decode | jq .id`
	echo -n "   "
	echo -n `echo $oarjob | base64 --decode | jq .name`
	echo -n "   "
	echo -n `echo $oarjob | base64 --decode | jq .state`
	echo -n "   "
	starting=`echo $oarjob | base64 --decode | jq .start_time`
	echo -n `date -d @"$starting"`
	echo -n "   "
	finishing=`echo $oarjob | base64 --decode | jq .walltime`
	total=$(( starting + finishing ))
	echo -n `date -d @"$total"`
	echo ""
done
