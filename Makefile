clippy:
	cargo clippy --workspace --tests

test:
	cargo test -- --nocapture --test-threads=1 $(ARGS)
