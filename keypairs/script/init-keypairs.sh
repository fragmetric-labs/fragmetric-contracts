#!/usr/bin/env bash

. ./init-env.sh

read -r -p "[?] Do you want to create/override all the keypair files? (y/n) " answer
if [[ $answer != [yY] ]]; then
    exit;
fi

$WALLET_KEYPAIR
  $RESTAKING_MINT_FRAGSOL_KEYPAIR
  $RESTAKING_DEVNET_PROGRAM_KEYPAIR
  $RESTAKING_DEVNET_ADMIN_KEYPAIR
  $RESTAKING_DEVNET_FUNDMANAGER_KEYPAIR
  $RESTAKING_MAINNET_PROGRAM_KEYPAIR
  $RESTAKING_MAINNET_ADMIN_KEYPAIR

echo "[*] Creating a placeholder keypair to $WALLET_KEYPAIR"
solana-keygen new -o mint_fragsol_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json

echo "create a placeholder keypair for devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP"
solana-keygen new -o devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json

echo "create a placeholder keypair for devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX"
solana-keygen new -o devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json

echo "create a placeholder keypair for devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ"
solana-keygen new -o devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json
