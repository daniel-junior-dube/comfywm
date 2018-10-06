use wlroots::{Area, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle};

use layout::Layout;

/*
.##...##...####...#####...##..##...####...#####....####....####...######.
.##...##..##..##..##..##..##.##...##......##..##..##..##..##..##..##.....
.##.#.##..##..##..#####...####.....####...#####...######..##......####...
.#######..##..##..##..##..##.##.......##..##......##..##..##..##..##.....
..##.##....####...##..##..##..##...####...##......##..##...####...######.
.........................................................................
*/

pub struct Workspace {
	pub window_layout: Layout,
}

impl Workspace {
	pub fn new(output_area: Area) -> Self {
		Workspace {
			window_layout: Layout::new(output_area),
		}
	}

	pub fn add_window(&mut self, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		self.window_layout.add_window(shell_handle);
	}
}
