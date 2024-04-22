debug_bin := ./target/debug/syn_port_scanner.exe
release_bin := ./target/release/syn_port_scanner.exe

build:
	@cargo build
	@cargo build --release

clean:
	@cargo clean