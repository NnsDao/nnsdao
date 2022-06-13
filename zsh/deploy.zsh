#!/bin/zsh
set -e

network=${1:-local}
canister=${2:-nnsdao}
mode=${3:-upgrade}
confirm_no_backup=${4:-false}

RED="\033[1;31m"
NOCOLOR="\033[0m"

# Confirm before deploying to mainnet
if [[ $network != "local" ]]
then
    echo "Confirm mainnet launch"
    select yn in "Yes" "No"; do
        case $yn in
            Yes ) break;;
            No ) exit;;
        esac
    done

    if [[ $confirm_no_backup != true ]]
    then
        echo "${RED}backuping.....${NOCOLOR}"
        # ./zsh/backup.zsh $network $canister
    fi
fi

if [[ $network == "local" ]]
then
    m="" && [[ $mode == "reinstall" ]] && m="-m reinstall"
    dfx deploy --network $network $m $canister
    exit 0
fi

echo "${RED}building.....${NOCOLOR}"
dfx build --network $network $canister 

echo "${RED}installing.....${NOCOLOR}"
dfx canister --network $network install $canister -m $mode 