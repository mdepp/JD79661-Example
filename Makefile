.phony: run-simulator
run-simulator:
	cargo run --package simulator

.phony: run-sundial
run-sundial:
	cargo run --package sundial --target thumbv6m-none-eabi
