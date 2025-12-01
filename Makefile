.phony: run-simulator
run-simulator:
	cargo run --bin simulator

.phony: run-sundial
run-sundial:
	cargo run --bin sundial --target thumbv6m-none-eabi
