#!/usr/bin/env bash
set -euo pipefail

HOST="127.0.0.1"

assert_status() {
  local method="$1"
  local port="$2"
  local host_header="$3"
  local path="$4"
  local expected_status="$5"

  local body_file
  body_file="$(mktemp)"

  local status
    status="$(curl -sS --connect-timeout 2 --max-time 10 -o "$body_file" -w "%{http_code}" -X "$method" -H "Host: $host_header" -H "Connection: close" "http://$HOST:$port$path")"

  if [[ "$status" != "$expected_status" ]]; then
    echo "[FAIL] $method $path on $host_header (port $port): expected $expected_status, got $status"
    echo "---- response body ----"
    cat "$body_file"
    echo
    rm -f "$body_file"
    exit 1
  fi

  echo "[PASS] $method $path on $host_header returned $status"
  rm -f "$body_file"
}

assert_body_contains() {
  local port="$1"
  local host_header="$2"
  local path="$3"
  local expected_substring="$4"

  local body_file
  body_file="$(mktemp)"

    curl -sS --connect-timeout 2 --max-time 10 -o "$body_file" -H "Host: $host_header" -H "Connection: close" "http://$HOST:$port$path"

  if ! grep -Fq "$expected_substring" "$body_file"; then
    echo "[FAIL] GET $path on $host_header did not contain expected text: $expected_substring"
    echo "---- response body ----"
    cat "$body_file"
    echo
    rm -f "$body_file"
    exit 1
  fi

  echo "[PASS] GET $path on $host_header body check"
  rm -f "$body_file"
}

echo "Running HTTP integration checks against running server..."

assert_status "GET" "8080" "localhost:8080" "/" "302"
assert_body_contains "8080" "localhost:8080" "/" "/login"
assert_status "GET" "8080" "localhost:8080" "/login" "200"
assert_status "GET" "8080" "localhost:8080" "/this-does-not-exist" "302"

assert_status "GET" "8081" "public:8081" "/hello" "200"
assert_body_contains "8081" "public:8081" "/hello" "SERVER_NAME=public"
assert_status "GET" "8081" "public:8081" "/this-does-not-exist" "404"

echo "All integration checks passed."
