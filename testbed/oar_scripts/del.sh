#!/usr/bin/env bash
for oarjob in `oarstat -u -f -J | jq -r ".[] | @base64"` ; do
	jobid=`echo $oarjob | base64 --decode | jq .id`
	echo -n $jobid
	echo -n "   "
	echo -n `echo $oarjob | base64 --decode | jq .name`
	echo -n "   "
	starting=`echo $oarjob | base64 --decode | jq .start_time`
	echo -n `date -d @"$starting"`
	echo -n "   "
	finishing=`echo $oarjob | base64 --decode | jq .walltime`
	total=$(( starting + finishing ))
	echo -n `date -d @"$total"`
	echo " --> deleted"
	oardel $jobid
done
