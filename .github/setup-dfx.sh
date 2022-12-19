#!/usr/bin/env bash

VERSION=$(node -p "require('./dfx.json').dfx")

if [[ "${VERSION}" != "undefined" ]]; then
  DFX_VERSION=${VERSION}
fi
echo "dfx version: ${DFX_VERSION}"

# //  Use the DFX_VERSION environment variable to identify a specific version of the SDK that you want to install.
sh -ci "$(curl -fsSL https://smartcontracts.org/install.sh)"
