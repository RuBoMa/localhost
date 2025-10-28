#!/usr/bin/env bash
set -euo pipefail

HOST="127.0.0.1"
PORT="8080"
ALT_PORT="8081"
SERVER_NAME="localhost"
BASE_URL="http://$HOST:$PORT"
HOMEPAGE_URL="$BASE_URL/"
IMAGE_URL="$BASE_URL/main"
REDIRECT_URL="$BASE_URL/old-page"
CGI_URL="$BASE_URL/hello"

run() {
    echo "$1"
    shift
    "$@"
    # Currently using sleep to be able to tell if the script is working as intended, remove later
    sleep 1
}

echo "Make sure that the program is already running in some terminal before tests start."
echo "Starting in 5"
sleep 1
echo "4"
sleep 1
echo "3"
sleep 1
echo "2"
sleep 1
echo "1"
sleep 1

run "Testing GET on homepage..." curl -i "$HOMEPAGE_URL"
run "Testing GET on image page..." curl -i "$IMAGE_URL"
run "Testing GET on redirect page..." curl -i "$REDIRECT_URL"
run "Testing GET on cgi page..." curl -i -H "Host: localhost:8080" "$CGI_URL"

if command -v siege >/dev/null 2>&1; then
  run "Running siege on homepage for 30 seconds..." siege -b "$HOMEPAGE_URL" -t 30s
else
  echo "[info] siege not installed; skipping stress test"
fi

sleep 30
