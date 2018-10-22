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
extern crate common;
extern crate libc;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::env;

use wlroots::utils::{init_logging as wlr_init_logging, WLR_DEBUG};

mod compositor;
mod input;
mod layout;

use compositor::generate_default_compositor;

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
	env_logger::init();
	wlr_init_logging(WLR_DEBUG, None);
	let compositor = generate_default_compositor();
	compositor.run()
}
