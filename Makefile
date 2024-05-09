debug_bin := .\target\debug\syn_port_scanner.exe
release_bin := .\target\release\syn_port_scanner.exe
packet_lib := .\lib\Packet.lib
wpcap_lib := .\lib\wpcap.lib

playground_lib_path := ..\play\lib
playground_packet_lib_path := ..\play\lib\Packet.lib
playground_wpcap_lib_path := ..\play\lib\wpcap.lib

debug_playground := ..\play\debug.exe
release_playground := ..\play\release.exe

build:
	@cargo build
	@cargo build --release
	@copy $(debug_bin) $(debug_playground)
	@copy $(release_bin) $(release_playground)
	@-mkdir $(playground_lib_path)
	@copy $(packet_lib) $(playground_packet_lib_path)
	@copy $(wpcap_lib) $(playground_wpcap_lib_path)

run: run-debug

run-debug:
	@cd .. && make run-debug

run-release:
	@cd .. && make run-release

clean:
	@cargo clean