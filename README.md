<div align="center">
	<img width="256px" src="./LOGO.png" alt="Comfy Logo">
</div>

## Comfy

Comfy is a Wayland compositor written in the Rust programming language and inspired by i3wm, bspwm and XMonad. It's main goal is ease of use, extensibility and a sane codebase.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.

### Prerequisites

What things you need to install the software and how to install them

```
meson
wayland
wayland-protocols
EGL
GLESv2
libdrm
GBM
libinput
xkbcommon
udev
pixman
systemd (optional, for logind support)
elogind (optional, for logind support on systems without systemd)
libcap (optional, for capability support)
xcb
xcb-composite
xcb-xfixes
xcb-image
xcb-render
x11-xcb
xcb-errors (optional, for improved error reporting)
x11-icccm (optional, for improved Xwayland introspection)
xcb-xkb (optional, for improved keyboard handling on the X11 backend)
```

### Developpment
A step by step series of examples that tell you how to get a development env running

**Arch Linux:**
```bash
...
```

**Ubuntu:**
```bash
...
```

**Debian:**
```bash
...
```

**Using Cargo:**
```bash
# Pull and update the git submodules
git submodule update --init --recursive

# Build the application
cargo build

# If the build was successfull, call the install command
cargo install
```

### Installation

A step by step series of examples that tell you how to install Comfywm on your system

**Arch Linux:**
```bash
...
```

**Ubuntu:**
```bash
...
```

**Debian:**
```bash
...
```

**Using Cargo:**
```bash
# Pull and update the git submodules
git submodule update --init --recursive

# Build the application
cargo build

# If the build was successfull, call the install command
cargo install
```

## Running the tests

Run the following command:

### All test:

```
cargo test
```

### A specific test:

```
cargo test <name of the test>
```

### Built With

* [Rust](https://www.rust-lang.org) - System programming language
* [wlroots](https://github.com/swaywm/wlroots) - A modular Wayland compositor library.
* [wlroots-rs](https://github.com/swaywm/wlroots-rs) - Safe Rust bindings for wlroots.

## Contributing

Please read [contributing](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning.

## Authors

* **Daniel-Junior Dubé** - *Initial work* - [Github](https://github.com/daniel-junior-dube), [Portfolio](https://daniel-junior-dube.github.io)
* **Félix Chabot** - *Initial work* - [Github](https://github.com/chabam)

See also the list of [contributors](CONTRIBUTORS.md) who participated in this project.

## License

This project is licensed under the MIT License - see the [license](LICENSE.md) file for details

## Acknowledgments

* Hat tip to anyone whose code was used
* Inspiration
* etc
