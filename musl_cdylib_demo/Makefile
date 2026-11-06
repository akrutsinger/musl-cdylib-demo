C_FLAGS := -Wall -Wextra -Wpedantic -std=c11
OUT := target/x86_64-unknown-linux-musl

# Phony so cargo always runs; it decides itself whether the .so is stale.
.PHONY: all release run clean

all:
	cargo build
	musl-gcc $(C_FLAGS) host/main.c -o $(OUT)/debug/host

release:
	cargo build --release
	musl-gcc $(C_FLAGS) host/main.c -o $(OUT)/release/host

run: all
	cd $(OUT)/debug && ./host

clean:
	cargo clean
