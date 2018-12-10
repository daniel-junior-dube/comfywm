CARGO_OPTS=

CARGO=cargo $(CARGO_OPTS)
SESSION=wayland-sessions

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
ifdef x11
	@echo "INSTALLING AS X11 SESSION"
	$(eval override SESSION=xsessions)
else
	@echo "INSTALLING AS WAYLAND SESSION"
endif
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
	cp -f data/comfy.desktop /usr/share/$(SESSION)

uninstall:
	rm -f /bin/comfywm
	rm -rf /etc/comfywm
	rm -f /usr/share/xsessions/comfy.desktop
	rm -f /usr/share/wayland-sessions/comfy.desktop
	rm -rf /usr/share/comfywm
