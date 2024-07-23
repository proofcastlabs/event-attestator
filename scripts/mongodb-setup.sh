#!/bin/bash

echo "Starting mongodb container on ${MONGO_HOST_PORT:-27017}"

docker run --name mongo -p ${MONGO_HOST_PORT:-27017}:27017 -d mongodb/mongodb-community-server:latest
