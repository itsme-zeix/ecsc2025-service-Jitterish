#!/usr/bin/env bash

set -e

# retain the latest $NUM users
NUM=512
# retain data directories for 120min
TIME=120

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
find data/db-engine-data/ -mindepth 1 -maxdepth 1 -type d -mmin +$TIME -not -name '_' -print0 | xargs -0 /bin/rm -rf

echo "Cleanup query cache..."
rm data/db-engine-queries/*.so

echo "Starting service..."
docker start jitterish-website-1

echo DONE