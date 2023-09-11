#!/bin/bash


docker build -t image/notifications-service .

docker run \
    --rm \
    --name notifications-service \
    -p 3000:3000 \
    image/notifications-service
