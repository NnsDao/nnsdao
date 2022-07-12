#!/usr/bin/env bash

# Make sure path ends with /
for directory in '.dfx/local/canisters/'*; do
  if [[ -d "${directory}" && ! -L "${directory}" ]]; then
    if [ -f "${directory}/index.js" ]; then
      # rename xxx/xxx.did.js  to xxx/xxx.ts
      mv ${directory}/${directory}.did.js ${directory}/${directory}.did.ts
      # rename xxx.did.d.ts  to types.ts
      mv ${directory}/${directory}.did.d.ts ${directory}/types.ts
    fi
  fi
done
