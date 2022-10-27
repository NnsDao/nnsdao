#!/usr/bin/env bash

# Make sure path ends with /
for directory in '.dfx/ic/canisters/'*; do
  if [[ -d "${directory}" && ! -L "${directory}" ]]; then

    if [ -f "${directory}/index.js" ]; then
      echo "${directory}"
      # rename xxx/xxx.did.js  to xxx/xxx.ts
      mv ${directory}/*.did.js ${directory}/index.ts
      # rename xxx.did.d.ts  to types.ts
      mv ${directory}/*.did.d.ts ${directory}/types.ts
    fi
  fi
done
