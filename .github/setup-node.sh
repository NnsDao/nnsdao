#!/usr/bin/env bash

echo "Install npm package"

if [ -f "./package.json" ]; then
  npm ls
else
  echo 'run npm init'
  npm init -y
fi

npm i @dfinity/agent @dfinity/principal @dfinity/candid @dfinity/identity glob -f

# for exist npm dependency
npm ci >/dev/null 2>&1 || true
