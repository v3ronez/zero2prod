#!/usr/bin/env bash
# set -x
set -eo pipefail

if docker compose ps --status running --format "{{.Name}}"| grep -qx "newsletter"; then
  echo >&2 "A docker container named 'newsletter' already running"
  exit 1
fi

docker compose up -d

until [ "$(docker compose exec -T db pg_isready -d postgres | awk -F'- ' '{print $2}')" = "accepting connections" ]; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 2
done

sqlx database create
sqlx migrate run
