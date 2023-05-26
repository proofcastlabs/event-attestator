#!/bin/bash
set -e
cd $(dirname -- $0)

BINARY_NAME=$1

echo [+] Cleaning up \'$1\'...

if [[ -d ../$BINARY_NAME/scripts/database ]]; then
	rm -r ../$BINARY_NAME/scripts/database
	echo [+] Test database dir cleaned up!
else
	echo [-] No database to clean up in \'$1\'
fi

if [[ -d ../$BINARY_NAME/scripts/logs ]]; then
	rm -r ../$BINARY_NAME/scripts/logs
	echo [+] Test logs dir cleaned up!
else
	echo [-] No logs to clean up in \'$1\'
fi

echo [+] Clean up done!
