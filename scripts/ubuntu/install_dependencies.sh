#/bin/bash

# Install from list of dependencies
apt-get install $(cat "./dependencies.txt")

# Compile/install wayland 1.15.0-2 from source
DESTINATION="wayland-latest"
wget "https://launchpad.net/ubuntu/+archive/primary/+sourcefiles/wayland/1.15.0-2/wayland_1.15.0.orig.tar.gz" -O ./$DESTINATION.tar.gz
tar -xf ./$DESTINATION.tar.gz --one-top-level=$DESTINATION --strip-components 1
pushd $DESTINATION
./configure
make
make install
popd
rm -rf ./$DESTINATION.tar.gz ./$DESTINATION

# Compile/install wayland-protocols 1.15.1 from source
DESTINATION="wayland-protocols-latest"
wget "https://launchpad.net/ubuntu/+archive/primary/+sourcefiles/wayland-protocols/1.15-1/wayland-protocols_1.15.orig.tar.xz" -O ./$DESTINATION.tar.gz
tar -xf ./$DESTINATION.tar.gz --one-top-level=$DESTINATION --strip-components 1
pushd $DESTINATION
./configure
make
make install
popd
rm -rf ./$DESTINATION.tar.gz ./$DESTINATION

