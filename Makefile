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

	# Copying the default configuration files to the system
	mkdir -p /etc/comfywm
	cp -f config/* /etc/comfywm/

	# Copying the wallpaper to the system
	mkdir -p /usr/share/comfywm/
	cp -f data/wallpaper.jpg /usr/share/comfywm/

	# Copying the desktop entry to the system
	cp -f data/comfy.desktop /usr/share/wayland-sessions

	# Copying the manual entry
	cp -f data/comfywm.1.gz /usr/share/man/man1/

uninstall:
	rm -f /bin/comfywm
	rm -rf /etc/comfywm
	rm -f /usr/share/wayland-sessions/comfy.desktop
	rm -rf /usr/share/comfywm
	rm -rf /usr/share/man/man1/comfywm.1.gz
