#!/usr/bin/env bash

# Make sure path ends with /
for directory in '.dfx/local/canisters/'*; do
  if [[ -d "${directory}" && ! -L "${directory}" ]]; then
    cp ${directory}/index.js ${directory}/index.d.ts
    echo ${directory}/index.d.ts
  fi
done
