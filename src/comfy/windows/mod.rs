use wlroots::{Area, Origin, Size, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle};

pub mod layout;

/*
.##...##..######..##..##..#####....####...##...##..#####....####...######...####..
.##...##....##....###.##..##..##..##..##..##...##..##..##..##..##....##....##..##.
.##.#.##....##....##.###..##..##..##..##..##.#.##..##..##..######....##....######.
.#######....##....##..##..##..##..##..##..#######..##..##..##..##....##....##..##.
..##.##...######..##..##..#####....####....##.##...#####...##..##....##....##..##.
..................................................................................
*/

#[derive(Clone)]
pub struct WindowData {
	pub shell_handle: WLRXdgV6ShellSurfaceHandle,
	pub area: Area,
}

impl WindowData {
	pub fn new(shell_handle: WLRXdgV6ShellSurfaceHandle) -> Self {
		WindowData {
			shell_handle,
			area: Area::new(Origin::new(0, 0), Size::new(0, 0)),
		}
	}
}
