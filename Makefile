clippy:
	cargo clippy --workspace --tests

tflags =

ifdef VERBOSE
    tflags += --nocapture --test-threads=1
endif

test:
	cargo test -- $(tflags) $(ARGS)

run:
	cargo run
