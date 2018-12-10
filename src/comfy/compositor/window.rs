use layout::LayoutDirection;
use utils::animation::Animation;
use utils::area_animation::AreaAnimation;
use utils::handle_helper;
use wlroots::utils::current_time;
use wlroots::{
	project_box as wlr_project_box, Area, Origin, Renderer as WLRRenderer, Size, SurfaceHandle as WLRSurfaceHandle,
	XdgV6ShellState as WLRXdgV6ShellState, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
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
	pub is_fullscreen: bool,
	current_area_animation: Option<AreaAnimation>,
	border_size: u8,
}

impl Window {
	pub fn new(shell_handle: WLRXdgV6ShellSurfaceHandle, area: Area, border_size: u8) -> Self {
		Window {
			shell_handle,
			area,
			is_fullscreen: false,
			current_area_animation: None,
			border_size,
		}
	}

	/// Creates a window with an empty area.
	pub fn new_empty_area(shell_handle: WLRXdgV6ShellSurfaceHandle, border_size: u8) -> Self {
		let area = Area::new(Origin::new(0, 0), Size::new(0, 0));
		Window::new(shell_handle, area, border_size)
	}

	pub fn render_top_level_surface(
		&self,
		renderer: &mut WLRRenderer,
		clear_color: &[f32; 4],
		cursor_orientation: Option<&LayoutDirection>,
		cursor_color: Option<&[f32; 4]>,
	) {
		let window_area = &self.area;
		self
			.shell_handle
			.run(|shell| {
				self.render_surface(
					renderer,
					&shell.surface(),
					window_area,
					0,
					0,
					clear_color,
					cursor_orientation,
					cursor_color,
				)
			}).unwrap();
	}

	pub fn render_all_surface(
		&self,
		renderer: &mut WLRRenderer,
		clear_color: &[f32; 4],
		cursor_orientation: Option<&LayoutDirection>,
		cursor_color: Option<&[f32; 4]>,
	) {
		let window_area = &self.area;
		self.for_each_surface(&mut |surface_handle: WLRSurfaceHandle, sx, sy| {
			self.render_surface(
				renderer,
				&surface_handle,
				&window_area,
				sx,
				sy,
				clear_color,
				cursor_orientation,
				cursor_color,
			);
		});
	}

	/// Renders the provided surface using the provided renderer. Also, renders the window borders and the cursor
	/// indicator
	#[wlroots_dehandle(surface)]
	fn render_surface(
		&self,
		renderer: &mut WLRRenderer,
		surface_handle: &WLRSurfaceHandle,
		window_area: &Area,
		sx: i32,
		sy: i32,
		clear_color: &[f32; 4],
		cursor_orientation_option: Option<&LayoutDirection>,
		cursor_color_option: Option<&[f32; 4]>,
	) {
		use surface_handle as surface;

		let border_offset = self.get_border_size();
		let total_border_offset = self.get_total_border_size();
		let (surface_width, surface_height) = surface.current_state().size();
		let transform = renderer.output.get_transform().invert();
		let output_transform_matrix = renderer.output.transform_matrix();
		let surface_size = Size::new(
			surface_width * renderer.output.scale() as i32,
			surface_height * renderer.output.scale() as i32,
		);

		// the render box is the whole subdivision of the screen (the shell including the Comfy's borders).
		let render_origin = Origin::new(window_area.origin.x + sx, window_area.origin.y + sy);
		let render_box = Area::new(render_origin, window_area.size);

		// We render the clear color under the window to create a border
		renderer.render_colored_rect(render_box, *clear_color, output_transform_matrix);

		// This section is used to render cursor indicator if needed (the window rendering right now is the active
		// window and is not fullscreen)
		if cursor_orientation_option.is_some() && !self.is_fullscreen {
			let cursor_orientation = cursor_orientation_option.unwrap();
			let mut cursor_indicator_offset_x = window_area.origin.x + sx;
			let mut cursor_indicator_offset_y = window_area.origin.y + sy;
			let mut cursor_indicator_width = border_offset;
			let mut cursor_indicator_height = border_offset;

			// We set accordingly the size and origin of the indicator
			if *cursor_orientation == LayoutDirection::Up || *cursor_orientation == LayoutDirection::Down {
				cursor_indicator_width = window_area.size.width;
				if *cursor_orientation == LayoutDirection::Down {
					cursor_indicator_offset_y += window_area.size.height - border_offset;
				}
			} else {
				cursor_indicator_height = window_area.size.height;
				if *cursor_orientation == LayoutDirection::Right {
					cursor_indicator_offset_x += window_area.size.width - border_offset
				}
			}

			let cursor_indicator_origin = Origin::new(cursor_indicator_offset_x, cursor_indicator_offset_y);
			let cursor_indicator_size = Size::new(cursor_indicator_width, cursor_indicator_height);
			let cursor_indicator_box = Area::new(cursor_indicator_origin, cursor_indicator_size);
			renderer.render_colored_rect(
				cursor_indicator_box,
				*cursor_color_option.unwrap(),
				output_transform_matrix,
			);
		}

		// The window_box is within the render_box, it is effectively the shell of the window we are rendering.
		let window_origin = Origin::new(
			window_area.origin.x + sx + border_offset,
			window_area.origin.y + sy + border_offset,
		);
		let window_box = Area::new(window_origin, surface_size);
		let window_matrix = wlr_project_box(window_box, transform, 0.0, output_transform_matrix);
		if let Some(texture) = surface.texture().as_ref() {
			// ? Restrict the render of the surface to the window's area if top level
			// TODO: Update set and clear scissor in a wrapper method
			if handle_helper::surface_helper::is_top_level(surface) {
				// We apply the scissor inside the borders
				let scissor_area = Area::new(
					Origin::new(self.area.origin.x + border_offset, self.area.origin.y + border_offset),
					Size::new(
						self.area.size.width - total_border_offset,
						self.area.size.height - total_border_offset,
					),
				);
				renderer.render_scissor(scissor_area);
				renderer.render_texture_with_matrix(texture, window_matrix);
				renderer.render_scissor(None);
			} else {
				renderer.render_texture_with_matrix(texture, window_matrix);
			}
		}
		surface.send_frame_done(current_time());
	}

	pub fn get_border_size(&self) -> i32 {
		if !self.is_fullscreen {
			self.border_size as i32
		} else {
			0
		}
	}

	pub fn get_total_border_size(&self) -> i32 {
		self.get_border_size() * 2
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
		self.current_area_animation = Some(AreaAnimation::new(
			self.area.clone(),
			destination,
			200,
			Animation::EaseInOutCirc,
		));
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
					let mut total_border_size = self.get_total_border_size() as u32;

					// This is an edge case where if we spawn too many windows the border would create a negative
					// window size.
					if total_border_size > self.area.size.width as u32 || total_border_size > self.area.size.height as u32 {
						total_border_size = 0;
					}
					toplevel.set_size(
						self.area.size.width as u32 - total_border_size,
						self.area.size.height as u32 - total_border_size,
					);
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
