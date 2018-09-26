use wlroots::{
	Capability, Compositor as WLRCompositor, CompositorBuilder as WLRCompositorBuilder, Cursor as WLRCursor,
	OutputLayout as WLROutputLayout, Seat, XCursorManager as WLRXCursorManager,
};

use input::{Cursor, InputManager};
use output::{OutputLayout, OutputManager};
use seat::SeatHandler;
use shell::XdgV6ShellManager;
use state::State;

/*
..####....####...##...##..#####....####....####...######..######...####...#####..
.##..##..##..##..###.###..##..##..##..##..##........##......##....##..##..##..##.
.##......##..##..##.#.##..#####...##..##...####.....##......##....##..##..#####..
.##..##..##..##..##...##..##......##..##......##....##......##....##..##..##..##.
..####....####...##...##..##.......####....####...######....##.....####...##..##.
.................................................................................
*/

pub fn generate_default_compositor() -> WLRCompositor {
	// ? WIP: Initialize the cursor structures
	let cursor = WLRCursor::create(Box::new(Cursor));
	let mut xcursor_manager =
		WLRXCursorManager::create("default".to_string(), 24).expect("Could not create xcursor manager");
	xcursor_manager.load(1.0);
	cursor
		.run(|c| xcursor_manager.set_cursor_image("left_ptr".to_string(), c))
		.unwrap();

	// ? WIP: Initialize the output layout structure
	let layout = WLROutputLayout::create(Box::new(OutputLayout));

	// ? WIP: Initialize the compositor structure
	let mut compositor = WLRCompositorBuilder::new()
		.gles2(true)
		.input_manager(Box::new(InputManager))
		.output_manager(Box::new(OutputManager))
		.xdg_shell_v6_manager(Box::new(XdgV6ShellManager))
		.build_auto(State::new(layout, xcursor_manager, cursor));

	// ? WIP: Initialize and add the seat structures to the state
	{
		let seat_handle = Seat::create(&mut compositor, "seat0".into(), Box::new(SeatHandler));
		seat_handle
			.run(|seat| {
				seat.set_capabilities(Capability::all());
			}).unwrap();
		let state: &mut State = (&mut compositor).into();
		state.seat_handle = Some(seat_handle);
	}

	compositor
}
