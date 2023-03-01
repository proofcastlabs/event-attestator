#!/bin/bash
set -e 
cd $(dirname -- $0)

BINARY_NAME=$1

if [[ -d ../$BINARY_NAME/scripts/database ]]; then
	rm -r ../$BINARY_NAME/scripts/database
fi

if [[ -d ../$BINARY_NAME/scripts/logs ]]; then
	rm -r ../$BINARY_NAME/scripts/logs
fi
