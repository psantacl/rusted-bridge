RUSTC ?= rustc

all: src/property_file.rs src/rusted_bridge.rs
		$(RUSTC) -o bin/wol --lib src/wol.rc
		$(RUSTC) -o bin/rusted-bridge  src/rusted_bridge.rs -L bin/

clean:
	rm -rf bin/*

install:
	cp bin/rusted-bridge ~/bin/

uninstall:
	rm ~/bin/rusted-bridge
