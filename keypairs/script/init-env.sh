#!/usr/bin/env bash

export BASE_DIR="$(realpath "$(dirname "$0")/../")"
export TARGET_PROGRAM_KEYPAIR_DIR="$(realpath "$BASE_DIR/../target/deploy")"

keypairs=(
  "WALLET_KEYPAIR=$BASE_DIR/wallet.json"
  "RESTAKING_MINT_FRAGSOL_KEYPAIR=$BASE_DIR/restaking/mint_fragsol_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json"
  "RESTAKING_DEVNET_PROGRAM_KEYPAIR=$BASE_DIR/restaking/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json"
  "RESTAKING_DEVNET_ADMIN_KEYPAIR=$BASE_DIR/restaking/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json"
  "RESTAKING_DEVNET_FUNDMANAGER_KEYPAIR=$BASE_DIR/restaking/devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json"
  "RESTAKING_MAINNET_PROGRAM_KEYPAIR=$BASE_DIR/restaking/mainnet_program_fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3.json"
  "RESTAKING_MAINNET_ADMIN_KEYPAIR=$BASE_DIR/restaking/mainnet_admin_fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby.json"
  "RESTAKING_TARGET_PROGRAM_KEYPAIR=$TARGET_PROGRAM_KEYPAIR_DIR/restaking-keypair.json"
)

export KEYPAIRS_STR="${keypairs[@]}"

get_keypair_path() {
    local search_key="$1"

    for keypair in "${keypairs[@]}"; do
        key="${keypair%%=*}"
        value="${keypair#*=}"
        if [[ "$key" == "$search_key" ]]; then
            echo "$value"
            return 0
        fi
    done

    echo "Key '$search_key' not found" >&2
    return 1
}

export -f get_keypair_path

echo "[*] Keypairs to be used:"
for keypair in "${keypairs[@]}"; do
    echo "$keypair"
done