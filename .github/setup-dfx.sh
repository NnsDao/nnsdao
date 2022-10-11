#!/usr/bin/env bash

DFX_VERSION=$(node -p "require('./dfx.json').dfx")

echo "dfx version: ${DFX_VERSION}"
# //  Use the DFX_VERSION environment variable to identify a specific version of the SDK that you want to install.
DFX_VERSION=${DFX_VERSION} sh -ci "$(curl -fsSL https://smartcontracts.org/install.sh)"
