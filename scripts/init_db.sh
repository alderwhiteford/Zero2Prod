#!/usr/bin/env bash
# allows for printing of each command executed for debugging
set -x
# -e causes the script to exit if any command fails
# -o pipefail causes the script to return a non-zero exit status if any pipeline command fails. Usually it is only
# the exit status of the last command in the pipelne that is considered (woohoo debugging!)
set -eo pipefail

# It is important to check if all the dependencies are installed before continuing with the script. 
# This will help avoid creating a broken state

# -x checks to see if the following command is executable. -v looks for the file path of the command
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it."
  exit 1
fi

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# Check if a custom password has been set, otherwise default to 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# Check if a custom database name has been set, otherwise default to 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# Check if a custom port has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  docker run \
      -e POSTGRES_USER=${DB_USER} \
      -e POSTGRES_PASSWORD=${DB_PASSWORD} \
      -e POSTGRES_DB=${DB_NAME} \
      -p "${DB_PORT}":5432 \
      -d postgres \
      postgres -N 1000 # Increased number of maximum connections for tests
fi

# Keep pinging Postgres until it's ready to accept commands
export PG_PASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  #>&2 redirects the output to stderr instead of stdout
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1 
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx migrate add create_subscriptions_table
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
