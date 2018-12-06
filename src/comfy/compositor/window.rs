use wlroots::{
	Area, Origin, Size, SurfaceHandle as WLRSurfaceHandle, XdgV6ShellState as WLRXdgV6ShellState,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};
use utils::animation::Animation;
use utils::area_animation::AreaAnimation;

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
	current_area_animation: Option<AreaAnimation>,
}

impl Window {
	pub fn new(shell_handle: WLRXdgV6ShellSurfaceHandle, area: Area) -> Self {
		Window { shell_handle, area, current_area_animation: None }
	}

	/// Creates a window with an empty area.
	pub fn new_empty_area(shell_handle: WLRXdgV6ShellSurfaceHandle) -> Self {
		let area = Area::new(Origin::new(0, 0), Size::new(0, 0));
		Window::new(shell_handle, area)
	}

	/// Sets the top level shell as maximized.
	pub fn set_maximized(&mut self) {
		self
			.shell_handle
			.run(|shell| {
				if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
					toplevel.set_maximized(true);
				}
			}).unwrap();
	}

	/// Sets the current animation from the current direction to a provided direction.
	/// Default animation options are:
	/// - 150ms duration
	/// - Ease-out-circ animation
	pub fn start_animation(&mut self, destination: Area) {
		self.current_area_animation = Some(
			AreaAnimation::new(self.area.clone(), destination, 150, Animation::EaseOutCirc)
		);
	}

	/// Progresses the current animation. Uses the delta time in the animation variant for a smooth progression.
	pub fn progress_animation_if_any(&mut self) {
		if self.current_area_animation.is_some() {
			let mut animation_has_ended = false;
			if let Some(ref mut current_area_animation) = self.current_area_animation {
				self.area = current_area_animation.current_area();
				if current_area_animation.has_ended() {
					animation_has_ended = true;
				}
			}
			self.apply_resize();
			if animation_has_ended {
				self.current_area_animation = None;
			}
		}
	}

	/// Sets the current size of the window and applies the area to the shell.
	pub fn resize(&mut self, new_area: Area) {
		self.area = new_area;
		self.apply_resize();
	}

	/// Applies the area to the top level shell.
	fn apply_resize(&mut self) {
		self
			.shell_handle
			.run(|shell| {
				if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
					toplevel.set_size(self.area.size.width as u32, self.area.size.height as u32);
				}
			}).unwrap();
	}

	/// Applies the provided function on all surfaces of the shell.
	pub fn for_each_surface(&self, f: &mut FnMut(WLRSurfaceHandle, i32, i32)) {
		self.shell_handle.run(|shell| shell.for_each_surface(f)).unwrap();
	}
}
