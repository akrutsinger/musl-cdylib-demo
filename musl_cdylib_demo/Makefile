C_FLAGS := -Wall -Wextra -Wpedantic -std=c11

.PHONY: all release run clean

all: target/x86_64-unknown-linux-musl/debug/host

release: target/x86_64-unknown-linux-musl/release/host

target/x86_64-unknown-linux-musl/debug/host: host/main.c
	cargo build --target x86_64-unknown-linux-musl
	musl-gcc $(C_FLAGS) host/main.c -o $@ -ldl

target/x86_64-unknown-linux-musl/release/host: host/main.c
	cargo build --target x86_64-unknown-linux-musl --release
	musl-gcc $(C_FLAGS) host/main.c -o $@ -ldl

run: all
	cd target/x86_64-unknown-linux-musl/debug && ./host

clean:
	cargo clean
