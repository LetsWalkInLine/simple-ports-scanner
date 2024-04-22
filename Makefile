debug_bin := .\target\debug\syn_port_scanner.exe
release_bin := .\target\release\syn_port_scanner.exe

debug_playground := ..\play\debug.exe
release_playground := ..\play\release.exe

build:
	@cargo build
	@cargo build --release
	@copy $(debug_bin) $(debug_playground)
	@copy $(release_bin) $(release_playground)

run: run-debug

run-debug:
	@cd .. && make run-debug

run-release:
	@cd .. && make run-release

clean:
	@cargo clean