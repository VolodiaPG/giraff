#!/usr/bin/env basha
set +x
file=$1
until [ -e $file ]; do
  sleep 2
  echo -n "."
done
echo ""
tail -f $file
