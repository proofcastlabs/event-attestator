#!/bin/bash

set -e

cd $(dirname -- $0)

PREPUSH_FILE=$(readlink -f ./pre-push)
PREPUSH_TARGET=$(readlink -f ../.git/hooks/pre-push)

if [[ -f $PREPUSH_TARGET ]] && ! [[ -v FORCE_REPLACE ]]; then
	echo "$PREPUSH_TARGET already exists, set FORCE_REPLACE env to replace"
elif [[ $PREPUSH_FILE -ef $PREPUSH_TARGET ]]; then
	echo "Link already set up"
else
	ln -fs $PREPUSH_FILE $PREPUSH_TARGET
	echo "Created link to pre-push hook"
fi

cd - > /dev/null
