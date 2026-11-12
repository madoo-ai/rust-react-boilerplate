#!/usr/bin/env bash
set -euo pipefail

# Playwright starts this script with DATABASE_URL pointing at the test
# database. Migrate, seed and serve so the browser suite never touches the
# development database.
cargo run -q -p rust-react-boilerplate-api -- migrate
cargo run -q -p rust-react-boilerplate-api -- seed
exec cargo run -q -p rust-react-boilerplate-api -- serve
