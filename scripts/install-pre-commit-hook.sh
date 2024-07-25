#!/bin/bash

set -e

cd $(dirname -- $0)

PRECOMMIT_FILE=$(readlink -f ./pre-commit)
PRECOMMIT_TARGET=$(readlink -f ../.git/hooks/pre-commit)

if [[ -f $PRECOMMIT_TARGET ]] && ! [[ -v FORCE_REPLACE ]]; then
	echo "$PRECOMMIT_TARGET already exists, set FORCE_REPLACE env to replace"
elif [[ $PRECOMMIT_FILE -ef $PRECOMMIT_TARGET ]]; then
	echo "Link already set up"
else
	ln -fs $PRECOMMIT_FILE $PRECOMMIT_TARGET
	echo "Created link to pre-commit hook"
fi

cd - > /dev/null
