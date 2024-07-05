#!/bin/bash

set -u -o pipefail

cd $(dirname -- $0)

echo "Building api app..."
docker build -t api-app ../api/
if (( $? )); then
  echo "Could not build api app, exiting..."
fi

api_app_container="api_app"
echo "Removing running api app, if any..."
docker kill $api_app_container 1>/dev/null 2>/dev/null
if (( $? )); then
  echo "Not running."
fi
docker container rm $api_app_container 1>/dev/null 2>/dev/null

echo "Checking envs..."
mongo_envs=(
  "MONGO_URI_STR"
  "MONGO_DB"
  "MONGO_COLLECTION"
)
mongo_envs_arg=""
for mongo_env in "${mongo_envs[@]}"
do
  if [ -v $mongo_env ]; then
    mongo_envs_arg="$mongo_envs_arg -e $mongo_env"
  else
    echo "$mongo_env env var is not set! Using default"
  fi
done

echo "Api app port set to ${API_APP_PORT:-44440}"

echo "Running api app..."
docker run -d --name $api_app_container -p ${API_APP_PORT:-44440}:80 --restart always $mongo_envs_arg api-app 1>/dev/null
echo "Done."

cd -
