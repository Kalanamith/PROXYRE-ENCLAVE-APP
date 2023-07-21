.PHONY: fmt
fmt:
	SKIP_WASM_BUILD= cargo fmt --all

.PHONY: lint
lint:
	SKIP_WASM_BUILD= cargo fmt --all -- --check
	SKIP_WASM_BUILD= cargo clippy --workspace -- -D dead_code -A clippy::derivable_impls -A clippy::explicit_counter_loop -A clippy::unnecessary_cast -A clippy::unnecessary_mut_passed -A clippy::too_many_arguments -A clippy::type_complexity -A clippy::identity_op -D warnings