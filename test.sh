#!/usr/bin/bash

curl \
    -X POST \
    -H 'Content-Type: application/json' \
    -d @download_request.json \
    http://127.0.0.1:8000/download

curl \
    -X POST \
    -H 'Content-Type: application/json' \
    -d @compile_request.json \
    http://127.0.0.1:8000/compile