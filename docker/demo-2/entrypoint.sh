#!/bin/sh
# entrypoint.sh

function terminate {
  echo "terminating"
  exit 0
}

trap terminate SIGTERM SIGINT

echo "reading $1"
tail -f $1 &
wait
