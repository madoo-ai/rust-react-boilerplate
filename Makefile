SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help

APP := rust-react-boilerplate-api
WEB := web
API_PORT ?= 3000

.PHONY: help setup dev api web build start lint test test-e2e fmt check-i18n themes \
	db-up db-migrate db-seed db-reset docker-build docker-up docker-down clean rename

help:
	@printf '%s\n' \
		'Rust React Boilerplate' \
		'' \
		'  make setup        create .env, install web deps, migrate and seed' \
		'  make dev          run the API and Vite together (API_PORT=3000)' \
		'  make api          run only the API on :3000' \
		'  make web          run only Vite on :5173' \
		'  make build        production web build and release binary' \
		'  make start        run the release binary' \
		'  make lint         fmt, clippy, ESLint, i18n and typecheck' \
		'  make test         Rust and web unit tests' \
		'  make test-e2e     Playwright against the test database' \
		'  make db-up        start Postgres and Mailpit with Compose' \
		'  make db-migrate   apply embedded migrations' \
		'  make db-seed      seed permissions, roles and demo users' \
		'  make db-reset     drop and rebuild the development schema' \
		'  make themes       regenerate themes.css and theme-options.ts' \
		'  make docker-up    run the full Compose stack' \
		'  make rename NAME="Client App" SLUG=client-app'

.env:
	@cp .env.example .env
	@secret=$$(openssl rand -hex 32); \
		sed -i "s/replace-with-at-least-32-characters/$$secret/" .env
	@echo "created .env with a generated APP_SECRET"

setup: .env
	@if command -v docker >/dev/null 2>&1 && ! pg_isready -h 127.0.0.1 -p 5432 -q 2>/dev/null; then \
		docker compose up -d postgres mailpit; \
	fi
	cd $(WEB) && npm install
	cargo build -p $(APP)
	$(MAKE) db-migrate
	$(MAKE) db-seed

dev:
	@trap 'kill 0' EXIT INT TERM; \
		PORT=$(API_PORT) cargo run -q -p $(APP) -- serve & \
		(cd $(WEB) && VITE_API_PROXY_TARGET=http://127.0.0.1:$(API_PORT) npm run dev) & \
		wait

api:
	PORT=$(API_PORT) cargo run -q -p $(APP) -- serve

web:
	cd $(WEB) && npm run dev

build:
	cd $(WEB) && npm run build
	cargo build --release -p $(APP)

start:
	./target/release/$(APP) serve

fmt:
	cargo fmt --all
	cd $(WEB) && npm run format

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -p $(APP) -- -D warnings
	cd $(WEB) && npm run lint
	cd $(WEB) && npm run typecheck
	cd $(WEB) && npm run check:i18n

test:
	cargo test -p $(APP)
	cd $(WEB) && npm test

test-e2e:
	cd $(WEB) && npx playwright test

check-i18n:
	cd $(WEB) && npm run check:i18n

themes:
	cd $(WEB) && npm run themes:generate

db-up:
	docker compose up -d postgres mailpit

db-migrate:
	cargo run -q -p $(APP) -- migrate

db-seed:
	cargo run -q -p $(APP) -- seed

db-reset:
	@set -a; . ./.env; set +a; \
		psql "$$DATABASE_URL" -v ON_ERROR_STOP=1 \
			-c 'DROP SCHEMA public CASCADE' \
			-c 'CREATE SCHEMA public'
	$(MAKE) db-migrate
	$(MAKE) db-seed

docker-build:
	docker compose build

docker-up:
	docker compose up --build

docker-down:
	docker compose down

clean:
	cargo clean
	rm -rf $(WEB)/dist $(WEB)/node_modules $(WEB)/coverage \
		$(WEB)/playwright-report $(WEB)/test-results

rename:
	@if [ -z "$(NAME)" ] || [ -z "$(SLUG)" ]; then \
		echo 'usage: make rename NAME="Client App" SLUG=client-app'; exit 1; \
	fi
	node scripts/rename.mjs "$(NAME)" "$(SLUG)"
