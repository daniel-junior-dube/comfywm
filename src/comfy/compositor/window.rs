use wlroots::utils::current_time;
use wlroots::{
	Area, Origin, Size, project_box as wlr_project_box, SurfaceHandle as WLRSurfaceHandle, XdgV6ShellState as WLRXdgV6ShellState,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle, Renderer as WLRRenderer
};
use utils::animation::Animation;
use utils::area_animation::AreaAnimation;
use utils::handle_helper;

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
	pub is_fullscreen: bool,
	current_area_animation: Option<AreaAnimation>,
}

impl Window {
	pub fn new(shell_handle: WLRXdgV6ShellSurfaceHandle, area: Area) -> Self {
		Window {
			shell_handle,
			area,
			is_fullscreen: false,
			current_area_animation: None,
		}
	}

	/// Creates a window with an empty area.
	pub fn new_empty_area(shell_handle: WLRXdgV6ShellSurfaceHandle) -> Self {
		let area = Area::new(Origin::new(0, 0), Size::new(0, 0));
		Window::new(shell_handle, area)
	}

	pub fn render_top_level_surface(&self, renderer: &mut WLRRenderer) {
		let window_area = &self.area;
		self.shell_handle.run(|shell| {
			self.render_surface(renderer, &shell.surface(), window_area, 0, 0)
		}).unwrap();
	}

	pub fn render_all_surface(&self, renderer: &mut WLRRenderer) {
		let window_area = &self.area;
		self.for_each_surface(&mut |surface_handle: WLRSurfaceHandle, sx, sy| {
			self.render_surface(renderer, &surface_handle, &window_area, sx, sy);
		});
	}

	/// Renders the provided surface using the provided renderer.
	#[wlroots_dehandle(surface)]
	fn render_surface(
		&self,
		renderer: &mut WLRRenderer,
		surface_handle: &WLRSurfaceHandle,
		window_area: &Area,
		sx: i32,
		sy: i32,
	) {
		use surface_handle as surface;
		let render_origin = Origin::new(window_area.origin.x + sx, window_area.origin.y + sy);
		let (width, height) = surface.current_state().size();
		let render_size = Size::new(
			width * renderer.output.scale() as i32,
			height * renderer.output.scale() as i32,
		);
		let render_box = Area::new(render_origin, render_size);
		let transform = renderer.output.get_transform().invert();
		let output_transform_matrix = renderer.output.transform_matrix();
		let matrix = wlr_project_box(render_box, transform, 0.0, output_transform_matrix);
		if let Some(texture) = surface.texture().as_ref() {
			// ? Restrict the render of the surface to the window's area if top level
			// TODO: Update set and clear scissor in a wrapper method
			if handle_helper::surface_helper::is_top_level(surface) {
				renderer.render_scissor(*window_area);
				renderer.render_texture_with_matrix(texture, matrix);
				renderer.render_scissor(None);
			} else {
				renderer.render_texture_with_matrix(texture, matrix);
			}
		}
		surface.send_frame_done(current_time());
	}

	pub fn has_active_animation(&self) -> bool {
		self.current_area_animation.is_some()
	}

	pub fn toggle_fullscreen(&mut self, is_fullscreen: bool) {
		self.is_fullscreen = is_fullscreen;
		self
			.shell_handle
			.run(|shell| {
				if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
					toplevel.set_fullscreen(is_fullscreen);
				}
			}).unwrap();
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
	/// - 200ms duration
	/// - Ease-in-out-circ animation
	pub fn start_animation(&mut self, destination: Area) {
		self.current_area_animation = Some(
			AreaAnimation::new(self.area.clone(), destination, 200, Animation::EaseInOutCirc)
		);
	}

	/// Progresses the current animation. Uses the delta time in the animation variant for a smooth progression.
	pub fn progress_animation(&mut self) {
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

	/// Send a close request to the top level shell.
	pub fn close(&self) {
		self
			.shell_handle
			.run(|shell| {
				if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
					toplevel.close();
				}
			}).unwrap();
	}

	/// Applies the provided function on all surfaces of the shell.
	pub fn for_each_surface(&self, f: &mut FnMut(WLRSurfaceHandle, i32, i32)) {
		self.shell_handle.run(|shell| shell.for_each_surface(f)).unwrap();
	}

	/// Convert the given output-related coordinates into window-related coordinates.
	pub fn convert_output_coord_to_window(&self, x: f64, y: f64) -> (f64, f64) {
		let local_x = x - (self.area.origin.x as f64);
		let local_y = y - (self.area.origin.y as f64);
		(local_x, local_y)
	}
}
