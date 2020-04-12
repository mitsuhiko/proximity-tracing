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

.PHONY: all doc test cargotest format format-check lint update-readme
