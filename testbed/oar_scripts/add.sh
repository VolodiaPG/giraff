#!/usr/bin/env bash
if [ "$#" -ne 1 ]
then
  echo "Usage: <param> additionnal hours for all jobs"
  exit 1
fi
additionnal=$1
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
	echo -n " --> "
	total=$(( starting + finishing + additionnal ))
	echo -n `date -d @"$total"`
	echo -n " --> "
	oarwalltime $jobid +"$additionnal:00"
done
