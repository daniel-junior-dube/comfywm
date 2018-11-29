use wlroots::{
	Area, Origin, Size, SurfaceHandle as WLRSurfaceHandle, XdgV6ShellState as WLRXdgV6ShellState,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

/*
.##...##..######..##..##..#####....####...##...##..#####....####...######...####..
.##...##....##....###.##..##..##..##..##..##...##..##..##..##..##....##....##..##.
.##.#.##....##....##.###..##..##..##..##..##.#.##..##..##..######....##....######.
.#######....##....##..##..##..##..##..##..#######..##..##..##..##....##....##..##.
..##.##...######..##..##..#####....####....##.##...#####...##..##....##....##..##.
..................................................................................
*/

#[derive(Clone)]
pub struct Window {
	pub shell_handle: WLRXdgV6ShellSurfaceHandle,
	pub area: Area,
}

impl Window {
	pub fn new(shell_handle: WLRXdgV6ShellSurfaceHandle, area: Area) -> Self {
		Window { shell_handle, area }
	}

	pub fn new_empty_area(shell_handle: WLRXdgV6ShellSurfaceHandle) -> Self {
		let area = Area::new(Origin::new(0, 0), Size::new(0, 0));
		Window::new(shell_handle, area)
	}

	pub fn set_maximized(&mut self) {
		self
			.shell_handle
			.run(|shell| {
				if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
					toplevel.set_maximized(true);
				}
			}).unwrap();
	}

	pub fn resize(&mut self, new_area: &Area) {
		self.area = *new_area;
		self
			.shell_handle
			.run(|shell| {
				if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
					toplevel.set_size(self.area.size.width as u32, self.area.size.height as u32);
				}
			}).unwrap();
	}

	pub fn for_each_surface(&self, f: &mut FnMut(WLRSurfaceHandle, i32, i32)) {
		self.shell_handle.run(|shell| shell.for_each_surface(f)).unwrap();
	}
}
