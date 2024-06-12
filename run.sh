#!/usr/bin/env bash

set -a
source .env
set +a

if [[ -z "$1" ]]; then
  cargo watch --quiet --exec 'run --release'
elif [[ "once" == "$1" ]]; then
  cargo run --release
elif [[ "deploy" == "$1" ]]; then
  if [[ "dev" == "$2" ]]; then
    DEPLOYMENT_HOST="dev-single.peoplesmarkets.com"
    DEPLOYMENT_ENV_FILE=".env.dev"
  elif [[ "prod" == "$2" ]]; then
    DEPLOYMENT_HOST="prod-single.peoplesmarkets.com"
    DEPLOYMENT_ENV_FILE=".env.prod"
  else
    echo "ERROR: Please provide environment to deploy to: 'dev' or 'prod'"
    exit
  fi

  cargo build --release
  ssh $DEPLOYMENT_HOST "sudo mkdir -p /opt/services/websites/"
  rsync --rsync-path='sudo rsync' websites.service $DEPLOYMENT_HOST:/etc/systemd/system/
  rsync --rsync-path='sudo rsync' $DEPLOYMENT_ENV_FILE $DEPLOYMENT_HOST:/opt/services/websites/.env
  rsync --rsync-path='sudo rsync' target/release/websites $DEPLOYMENT_HOST:/opt/services/websites/
  ssh $DEPLOYMENT_HOST "sudo chmod +x /opt/services/websites/websites && sudo systemctl daemon-reload && sudo systemctl restart websites.service"
else
  echo "ERROR: Unknown parameter '$1'"
fi
