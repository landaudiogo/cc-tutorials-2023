#!/bin/bash

set -e


USAGE="
Usage: run.sh <topic>

    topic: The topic where messages are to be produced.
"

if ! (( $# > 0 )); then
    echo $USAGE
    exit -1
fi

topic="$1"

docker build -t tkafka/simple simple_producer_consumer

docker run \
    --rm \
    -d \
    --name simple_consumer \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/simple consumer.py "$topic"

docker run \
    --rm \
    -it \
    --name simple_producer \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/simple producer.py "$topic"
