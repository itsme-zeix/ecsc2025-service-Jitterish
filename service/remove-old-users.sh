#!/usr/bin/env bash

set -e

# WHY: service might get slow if database is too large.
# WHAT: this script removes old users and their databases
# USAGE: run on vulnbox, not in docker.

# retain the latest $NUM users
NUM=750
# retain data directories for 120min
TIME=120

cd "$(dirname "$0")"

echo "Stopping service..."
docker stop jitterish-website-1

echo "Cleanup main database..."
tail -$NUM data/db-engine-data/_/users.ndjson > data/db-engine-data/_/users.ndjson.tmp
rm data/db-engine-data/_/users.ndjson
mv data/db-engine-data/_/users.ndjson.tmp data/db-engine-data/_/users.ndjson
tail -$NUM data/db-engine-data/_/complaints.ndjson > data/db-engine-data/_/complaints.ndjson.tmp
rm data/db-engine-data/_/complaints.ndjson
mv data/db-engine-data/_/complaints.ndjson.tmp data/db-engine-data/_/complaints.ndjson

echo "Cleanup old user directories..."
find data/db-engine-data/ -mindepth 1 -maxdepth 1 -type d -mmin +$TIME -not -name '_' -not -name 'demo_community' -not -name 'demo_enterprise' -print0 | xargs -0 /bin/rm -rf

echo "Cleanup query cache..."
rm data/db-engine-queries/*.so

echo "Starting service..."
docker start jitterish-website-1

echo DONE