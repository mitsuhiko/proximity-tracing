all: test

build:
	@cargo build

doc:
	@cargo doc

test: cargotest

cargotest:
	@cd contact-tracing; cargo test 
	@cd contact-tracing; cargo check --no-default-features
	@cd contact-tracing; cargo test --all-features
	@cd backend-service; cargo check

format:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt --all

format-check:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt --all -- --check

lint:
	@rustup component add clippy 2> /dev/null
	@cargo clippy --all

update-readme:
	@cd contact-tracing; cargo readme | perl -p -e "s/\]\(([^\/]+)\)/](https:\/\/docs.rs\/contact-tracing\/latest\/contact_tracing\/\\1)/" > README.md

server:
	@cd backend-service; RUST_LOG=debug cargo run

server-reload:
	@cd backend-service; RUST_LOG=debug systemfd --no-pid -s http::5000 -- cargo watch -x run

.PHONY: all doc test cargotest format format-check lint update-readme server server-reload
