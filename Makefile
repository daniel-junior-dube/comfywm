CARGO_OPTS =

CARGO = cargo $(CARGO_OPTS)

all:
	$(MAKE) build

build:
	$(CARGO) build --release

clean:
	$(CARGO) clean

test:
	$(MAKE) build
	$(CARGO) test

bench:
	$(CARGO) bench

doc:
	$(CARGO) doc

# Requires sudo rights
install:
	# Copying the binairies files to the system
	cp -f target/release/comfywm /bin
	chmod 755 /bin/comfywm

	# Copying the configuration files to the system
	mkdir -p /etc/comfywm
	cp -f config/keybindings.toml config/theme.toml /etc/comfywm/

	# Copying the desktop entry to the system
	cp -f data/comfy.desktop /usr/share/wayland-sessions

uninstall:
	rm -f /bin/comfywm
	rm -rfb /etc/comfywm
	rm -f /usr/share/wayland-sessions/comfy.desktop
