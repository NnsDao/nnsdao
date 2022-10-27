#!/usr/bin/env bash

DFX_VERSION=$(node -p "require('./dfx.json').dfx")

echo "Install dfx,version: ${DFX_VERSION}"
# //  Use the DFX_VERSION environment variable to identify a specific version of the SDK that you want to install.
DFX_VERSION=${DFX_VERSION} sh -ci "$(curl -fsSL https://smartcontracts.org/install.sh)"

echo "Install global npm package"

if [ -f "./package.json" ]; then
  echo "File \"./package.json\" exists"
else
  echo 'Init npm project config'
  npm init -y
fi

npm i @dfinity/agent @dfinity/principal @dfinity/candid @dfinity/identity glob -f

# npm list -g --depth=0
