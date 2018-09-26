use wlroots::{
	CursorHandle as WLRCursorHandle, KeyboardHandle as WLRKeyboardHandle, OutputLayoutHandle as WLROutputLayoutHandle,
	SeatHandle as WLRSeatHandle, XCursorManager as WLRXCursorManager,
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
	pub xcursor_manager: WLRXCursorManager,
	pub cursor: WLRCursorHandle,
	pub keyboard: Option<WLRKeyboardHandle>,
	pub layout: WLROutputLayoutHandle,
	pub shells: Vec<WLRXdgV6ShellSurfaceHandle>,
	pub seat_handle: Option<WLRSeatHandle>,
	pub current_rotation: f32,
}

impl State {
	pub fn new(layout: WLROutputLayoutHandle, xcursor_manager: WLRXCursorManager, cursor: WLRCursorHandle) -> Self {
		State {
			xcursor_manager,
			cursor,
			keyboard: None,
			layout: layout,
			shells: vec![],
			seat_handle: None,
			current_rotation: 0.0,
		}
	}
}

compositor_data!(State);
