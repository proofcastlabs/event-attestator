#!/bin/bash

#
# Author: gitmp01 (mauro@oraclize.it)
# Version: 1.0.1
#

if [[ -z "$FOLDER_CARGO" ]]; then
	FOLDER_CARGO=$HOME/.cargo
fi

if [[ -z "$DEBUG" ]]; then
	DEBUG=0
fi

if [[ -z "$DOCKER_EXC" ]]; then
	DOCKER_EXC=$(which docker)
	if [[ -z "$DOCKER_EXC" ]]; then
		echo 'Error: docker binary not found, submit the path through the DOCKER_EXC varieble'
		exit 1
	fi
fi

function logd() {
	if [[ "$DEBUG" != "0" ]]; then
		echo "$1"
	fi
}

function check_folder_exists() {
	local folder
	folder=$1
	if [[ ! -d "$folder" ]]; then
		echo "Error: folder $folder doesn't exist"
		exit 1
	fi
}

function check_image_exists() {
	docker image ls | grep "$1" > /dev/null
}

function maybe_build_image() {
	local image_name
	image_name=$1

	if [[ -n "$REBUILD" ]]; then
		docker image rm "$image_name"
	fi

	if ! check_image_exists "$image_name"; then
		logd "Image does not exist"
		docker build -t "$image_name" .
	fi
}

function maybe_add_env_variable() {
	local command

	__out=$1
	command=$2
	env_name=$3
	env_value=$4

	if [[ -z "$env_value" ]]; then
		out=$command
	else
		out="$command -e $env_name=$env_value"
	fi

	eval "$__out"="'$out'"
}

function cargo_build() {
	local cmd
	cmd="$DOCKER_EXC run"
	cmd="$cmd -ti --rm --name app"
	cmd="$cmd --volume $(pwd):/usr/src/myapp"
	cmd="$cmd --volume $FOLDER_CARGO/git:/usr/local/cargo/git"
	cmd="$cmd --volume $FOLDER_CARGO/env:/usr/local/cargo/env"
	cmd="$cmd --volume $FOLDER_CARGO/registry:/usr/local/cargo/registry"

	maybe_add_env_variable cmd "$cmd" "LOG_LEVEL" "$LOG_LEVEL"
	maybe_add_env_variable cmd "$cmd" "NUM_LOGS" "$NUM_LOGS"

	cmd="$cmd vanilla-apps $@"

	logd "$cmd"

	eval "$cmd"
}

function usage() {
  local b=$(tput bold)
  local n=$(tput sgr0)

  echo "${b}Usage:${n} $(basename "$0") <cargo-args>"
  echo ""
  echo "${b}Description:${n}"
  echo " Helper script to build the correct binary ready to be deployed in production."
  echo ""
  echo ""
  echo "${b}Examples:${n}"
  echo ""
  echo "Build the binary:"
  echo ""
  echo "  ./$(basename "$0") build --release --bin pbtc-on-int"
  echo ""
  echo "Show debug logging:"
  echo ""
  echo "  DEBUG=1 ./$(basename "$0")"
  echo ""
  echo "Rebuild the image (useful when updating the Dockerfile)"
  echo ""
  echo "  REBUILD=1 ./$(basename "$0")"
 }

function main() {
	check_folder_exists "$FOLDER_CARGO"

	if [[ -z "$1" ]]; then
		usage
		exit 1
	fi

	maybe_build_image "vanilla-apps"
	cargo_build "$@"
}


main "$@"
