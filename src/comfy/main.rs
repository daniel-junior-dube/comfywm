/*

MM'""""'YMM MMP"""""YMM M"""""`'"""`YM MM""""""""`M M""MMMM""M M""MMM""MMM""M M"""""`'"""`YM
M' .mmm. `M M' .mmm. `M M  mm.  mm.  M MM  mmmmmmmM M. `MM' .M M  MMM  MMM  M M  mm.  mm.  M
M  MMMMMooM M  MMMMM  M M  MMM  MMM  M M'      MMMM MM.    .MM M  MMP  MMP  M M  MMM  MMM  M
M  MMMMMMMM M  MMMMM  M M  MMM  MMM  M MM  MMMMMMMM MMMb  dMMM M  MM'  MM' .M M  MMM  MMM  M
M. `MMM' .M M. `MMM' .M M  MMM  MMM  M MM  MMMMMMMM MMMM  MMMM M  `' . '' .MM M  MMM  MMM  M
MM.     .dM MMb     dMM M  MMM  MMM  M MM  MMMMMMMM MMMM  MMMM M    .d  .dMMM M  MMM  MMM  M
MMMMMMMMMMM MMMMMMMMMMM MMMMMMMMMMMMMM MMMMMMMMMMMM MMMMMMMMMM MMMMMMMMMMMMMM MMMMMMMMMMMMMM

	Authors:
		- Daniel-Junior Dubé
		- Félix Chabot
	Date:
		September 2018
*/
#[macro_use]
extern crate wlroots;
extern crate chrono;
extern crate common;
extern crate libc;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate image;

use wlroots::utils::{init_logging as wlr_init_logging, WLR_DEBUG};

mod compositor;
mod config;
mod input;
mod layout;
mod utils;

use compositor::generate_default_compositor;
use utils::logger::{generate_log4rs_config, generate_wlroots_rs_log_callback};

/*
.##.....##....###....####.##....##
.###...###...##.##....##..###...##
.####.####..##...##...##..####..##
.##.###.##.##.....##..##..##.##.##
.##.....##.#########..##..##..####
.##.....##.##.....##..##..##...###
.##.....##.##.....##.####.##....##
*/

fn main() {
	// ? WIP: Required for x application to start, will be dynamically set if we wish to keep xwayland
	// TODO: env::set_var("DISPLAY", ":1");
	let log4rs_config = generate_log4rs_config();
	// ? Use this handle to edit logging at runtime
	let _handle = log4rs::init_config(log4rs_config).unwrap();
	wlr_init_logging(WLR_DEBUG, generate_wlroots_rs_log_callback());
	let compositor = generate_default_compositor();
	compositor.run()
}
