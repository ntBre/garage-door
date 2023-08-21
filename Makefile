clippy_flags =
ifdef DENY
    clippy_flags += -- -Dwarnings
endif

clippy:
	cargo clippy --workspace --tests $(clippy_flags)

tflags =

ifdef VERBOSE
    tflags += --nocapture --test-threads=1
endif

test:
	cargo test -- $(tflags) $(ARGS)

run:
	cargo run -- $(ARGS)

convert:
	cargo run -- convert		\
	    testfiles/core-opt.json	\
	    --dataset-type Optimization
