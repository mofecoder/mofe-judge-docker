#!/usr/bin/bash

CONTAINER_IP='127.0.0.1'

curl \
    -X POST \
    -H 'Content-Type: application/json' \
    -d @download_request.json \
    http://$CONTAINER_IP:8000/download

curl \
    -X POST \
    -H 'Content-Type: application/json' \
    -d @compile_request.json \
    http://$CONTAINER_IP:8000/compile

curl \
    -X POST \
    -H 'Content-Type: application/json' \
    -d @judge_request.json \
    http://$CONTAINER_IP:8000/judge