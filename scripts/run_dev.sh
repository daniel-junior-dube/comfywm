#!/bin/bash

pushd ..

# Start the window manager
cargo run &

sleep 3

# Start `gnome-terminal` inside the window manager
WAYLAND_DISPLAY=wayland-0 gnome-terminal

popd
