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
extern crate libc;

use wlroots::utils::{init_logging as wlr_init_logging, WLR_DEBUG};
use wlroots::{
	Capability, CompositorBuilder as WLRCompositorBuilder, OutputManagerHandler as WLROutputManagerHandler, Renderer,
	Seat, SeatHandle as WLRSeatHandle, SeatHandler as WLRSeatHandler,
};

mod input;
mod output;
mod seat;
mod shell;
mod state;
mod surface;

use input::InputManager;
use output::OutputManager;
use seat::SeatHandler;
use shell::XdgV6ShellManager;
use state::State;

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
	wlr_init_logging(WLR_DEBUG, None);

	let mut compositor = WLRCompositorBuilder::new()
		.gles2(true)
		.input_manager(Box::new(InputManager))
		.output_manager(Box::new(OutputManager))
		.xdg_shell_v6_manager(Box::new(XdgV6ShellManager))
		.data_device(true)
		.build_auto(State::new());

	{
		let seat_handle = Seat::create(&mut compositor, "seat0".into(), Box::new(SeatHandler));
		seat_handle
			.run(|seat| {
				seat.set_capabilities(Capability::all());
			}).unwrap();
		let state: &mut State = (&mut compositor).into();
		state.seat_handle = Some(seat_handle);
	}

	compositor.run()
}
