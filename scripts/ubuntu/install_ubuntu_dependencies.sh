#/bin/bash

# TODO: Replace list of dependencies with link to file (dependencies/ubuntu/dependencies.txt)
FILENAME="./dependencies.txt"
apt-get install $(grep -vE "^\s*#" $FILENAME  | tr "\n" " ")

# Compile/install wayland 1.15.0-2 from source
wget "https://launchpad.net/ubuntu/+archive/primary/+sourcefiles/wayland/1.15.0-2/wayland_1.15.0.orig.tar.gz" -O ./wayland-latest.tar.gz
tar -xf ./wayland-latest.tar.gz
pushd wayland-latest
./configure
make
make install
popd
rm -rf ./wayland-latest.tar.gz ./wayland-latest

# Compile/install wayland-protocols 1.15.1 from source
wget "https://launchpad.net/ubuntu/+archive/primary/+sourcefiles/wayland-protocols/1.15-1/wayland-protocols_1.15.orig.tar.xz" -O ./wayland-protocols-latest.tar.gz
tar -xf ./wayland-protocols-latest.tar.gz
pushd wayland-protocols-latest
./configure
make
make install
popd
rm -rf ./wayland-protocols-latest.tar.gz ./wayland-protocols-latest

