#!/bin/bash
cd $(dirname -- $0)

# NOTE: These are the branches we care to keep up to date, but which can't be merged into master
# for whatever reasons. So instead we check that they're rebased on pushes to master via GH actions.
branches=("pltc" "pdoge" "prvn" "plbry")

exitCode=0

currentCommitHash=$(git rev-parse HEAD)

for branch in "${branches[@]}" ; do
  git --no-pager branch -a --contains $currentCommitHash | grep -i $branch >> /dev/null
  gitCommandExitCode=$?
  if [ $gitCommandExitCode -ne 0 ]
  then
    echo ✘ The \'$branch\' branch has NOT been rebased on to this one!
    exitCode=1
  else
    echo ✔ The \'$branch\' branch HAS been rebased on to this one!
  fi
done

if [ $exitCode -ne 0 ]
then
  echo ✘ Branch rebase checks failed!
  exit $exitCode
else
  echo ✔ Branch rebase checks passed!
  exit 0
fi
