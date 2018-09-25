use wlroots::{
	KeyboardHandle as WLRKeyboardHandle, SeatHandle as WLRSeatHandle,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

/*
..####...######...####...######..######.
.##........##....##..##....##....##.....
..####.....##....######....##....####...
.....##....##....##..##....##....##.....
..####.....##....##..##....##....######.
........................................
*/

pub struct State {
	pub keyboard: Option<WLRKeyboardHandle>,
	pub shells: Vec<WLRXdgV6ShellSurfaceHandle>,
	pub seat_handle: Option<WLRSeatHandle>,
	pub current_rotation: f32,
}

impl State {
	pub fn new() -> Self {
		State {
			keyboard: None,
			shells: vec![],
			seat_handle: None,
			current_rotation: 0.0,
		}
	}
}

compositor_data!(State);
