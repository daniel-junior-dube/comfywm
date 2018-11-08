use wlroots::{Area, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle};

use compositor::window::Window;
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
}
