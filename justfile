set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Format the codebase using nightly cargo
fmt:
    cargo +nightly fmt --all -- --config format_code_in_doc_comments=true

# Check the codebase for errors
check:
    cargo check --all

# Check the codebase using clippy
clippy:
    cargo clippy --all

# Run the tests for the codebase
test:
    cargo test --all

# Run all pre-commit hooks
precommit: fmt check clippy test

bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p negentropy

graph:
	@cargo flamegraph --version || cargo install flamegraph
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p perf -o flamegraph.svg
