use wlroots::{
	Area, CompositorHandle as WLRCompositorHandle, Origin, OutputBuilder as WLROutputBuilder,
	SurfaceHandle as WLRSurfaceHandle, OutputBuilderResult as WLROutputBuilderResult, OutputHandle as WLROutputHandle, OutputHandler as WLROutputHandler,
	OutputLayoutHandler as WLROutputLayoutHandler, /* , OutputDestruction as WLROutputDestruction */
	OutputManagerHandler as WLROutputManagerHandler, Size,
};

use common::colors::Color;
use compositor::workspace::Workspace;
use compositor::ComfyKernel;
use compositor::window::Window;

/*
..####...##..##..######..#####...##..##..######..#####....####...######...####..
.##..##..##..##....##....##..##..##..##....##....##..##..##..##....##....##..##.
.##..##..##..##....##....#####...##..##....##....##..##..######....##....######.
.##..##..##..##....##....##......##..##....##....##..##..##..##....##....##..##.
..####....####.....##....##.......####.....##....#####...##..##....##....##..##.
................................................................................
*/

pub struct OutputData {
	pub workspace: Workspace,
	pub clear_color: [f32; 4],
}

impl OutputData {
	/// Creates data for an output which contains a workspace with a layout with the given area.
	pub fn new(area: Area) -> Self {
		OutputData {
			workspace: Workspace::new(area),
			clear_color: Color::burgundy().as_rgba_slice(),
		}
	}

	pub fn get_active_window(&self) -> Option<Window> {
		self.workspace.window_layout.get_active_window()
	}

	fn get_window_at(&self, x: f64, y: f64) -> Option<Window> {
		self.workspace.window_layout.find_window_at(x, y)
	}

	/// Return a tuple for a window which contains a subsurface that intersects with the provided absolute coordinates.
	/// Checks the active window first, since there is more chances that it intersects.
	/// This is mainly used to make sure we send pointer event relative to the surface under the pointer.
	// TODO: Maybe we should separate this into 2 separate functions. One that checks the active window and the other, the get_window_at...
	pub fn get_window_and_subsurface_at(&self, x: f64, y: f64) -> Option<(Window, WLRSurfaceHandle, f64, f64)> {
		let mut subsurface_intersection_at = None;
		if let Some(window) = self.get_active_window() {
			let (window_x, window_y) = window.convert_output_coord_to_window(x, y);
			if let Some((surface_handle, sx, sy)) = window.get_subsurface_at(window_x, window_y) {
				subsurface_intersection_at = Some((window, surface_handle, sx, sy));
			}
		}
		if subsurface_intersection_at.is_none() {
			if let Some(window) = self.get_window_at(x, y) {
				let (window_x, window_y) = window.convert_output_coord_to_window(x, y);
				if let Some((surface_handle, sx, sy)) = window.get_subsurface_at(window_x, window_y) {
					subsurface_intersection_at = Some((window, surface_handle, sx, sy));
				}
			}
		}
		subsurface_intersection_at
	}
}

/*
..####...##..##..######..#####...##..##..######..........##.......####...##..##...####...##..##..######.
.##..##..##..##....##....##..##..##..##....##............##......##..##...####...##..##..##..##....##...
.##..##..##..##....##....#####...##..##....##............##......######....##....##..##..##..##....##...
.##..##..##..##....##....##......##..##....##............##......##..##....##....##..##..##..##....##...
..####....####.....##....##.......####.....##............######..##..##....##.....####....####.....##...
........................................................................................................
.##..##...####...##..##..#####...##......######..#####..
.##..##..##..##..###.##..##..##..##......##......##..##.
.######..######..##.###..##..##..##......####....#####..
.##..##..##..##..##..##..##..##..##......##......##..##.
.##..##..##..##..##..##..#####...######..######..##..##.
........................................................
*/

// ? Handles events on the output layout (how displays are organized)
pub struct OutputLayoutHandler;
impl WLROutputLayoutHandler for OutputLayoutHandler {}

// ? Handles the actions and events of a specific output
pub struct OutputHandler;
impl WLROutputHandler for OutputHandler {
	/// Called every time the output frame is updated.
	#[wlroots_dehandle(compositor, output)]
	fn on_frame(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		use compositor_handle as compositor;
		use output_handle as output;
		let output_name = output.name().clone();
		let (output_width, output_height) = output.effective_resolution();
		let mut transform_matrix = output.transform_matrix();
		let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
		let renderer = compositor
			.renderer
			.as_mut()
			.expect("Compositor was not loaded with a renderer");
		let mut render_context = renderer.render(output, None);

		// ? Clearing the screen and get indices of windows to render
		let wallpaper_option = &comfy_kernel.wallpaper_texture;
		let active_color = &comfy_kernel.config.theme.active_color.as_slice();
		let inactive_color = &comfy_kernel.config.theme.inactive_color.as_slice();
		let cursor_indicator_color = &comfy_kernel.config.theme.cursor_indicator_color.as_slice();
		let cursor_orentation = comfy_kernel.cursor_direction.clone();
		if let Some(OutputData {
			workspace, clear_color, ..
		}) = comfy_kernel.output_data_map.get_mut(&output_name)
		{
			// ? Clear the screen with an image or the render color otherwise
			if let Some(wallpaper_texture) = wallpaper_option {
				let (texture_width, texture_height) = wallpaper_texture.size();
				let (scale_x, scale_y) = (
					output_width as f32 / texture_width as f32,
					output_height as f32 / texture_height as f32,
				);
				transform_matrix[0] = transform_matrix[0] * scale_x;
				transform_matrix[4] = transform_matrix[4] * scale_y;
				render_context.render_texture(&wallpaper_texture, transform_matrix, 0, 0, 1.0);
			} else {
				render_context.clear(clear_color.clone());
			}

			// ? Renders all windows
			if !workspace.window_layout.should_only_render_active_window() {
				workspace.window_layout.for_each_non_active_window(|window_ref| {
					if window_ref.has_active_animation() {
						window_ref.progress_animation();
					}
					window_ref.render_all_surfaces(&mut render_context, inactive_color, None, None);
				});
			}

			workspace.window_layout.apply_to_active_window(|window_ref| {
				if window_ref.has_active_animation() {
					window_ref.progress_animation();
				}
				window_ref.render_all_surfaces(
					&mut render_context,
					active_color,
					Some(&cursor_orentation),
					Some(cursor_indicator_color)
				);
			});
		}
	}

	/// WIP
	/// Called every time the output frame is updated.
	fn on_transform(&mut self, _: WLRCompositorHandle, _: WLROutputHandle) {
		debug!("on_transform");
	}

	/// Called every time the output mode changes.
	#[wlroots_dehandle(compositor, output)]
	fn on_mode_change(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		use compositor_handle as compositor;
		use output_handle as output;
		let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
		let output_data_map = &mut comfy_kernel.output_data_map;
		if let Some(output_data) = output_data_map.get_mut(&output.name()) {
			let (x, y) = output.layout_space_pos();
			let (width, height) = output.effective_resolution();
			output_data
				.workspace
				.window_layout
				.update_area_and_rebalance(Area::new(Origin::new(x, y), Size::new(width, height)));
		}
	}

	/// Called every time the output is enabled.
	fn on_enable(&mut self, _: WLRCompositorHandle, _: WLROutputHandle) {
		println!("on_enable");
	}

	/// Called every time the output scale changes.
	fn on_scale_change(&mut self, _: WLRCompositorHandle, _: WLROutputHandle) {
		println!("on_scale_change");
	}

	/// Called every time the buffers are swapped on an output.
	fn on_buffers_swapped(&mut self, _: WLRCompositorHandle, _: WLROutputHandle) {
		//println!("on_buffers_swapped");
	}

	/// Called every time the buffers need to be swapped on an output.
	fn needs_swap(&mut self, _: WLRCompositorHandle, _: WLROutputHandle) {
		//println!("needs_swap");
	}

	/// Called when an output is destroyed (e.g. unplugged).
	#[wlroots_dehandle(compositor, output)]
	fn destroyed(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		use compositor_handle as compositor;
		use output_handle as output;
		let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
		let output_name = output.name();
		info!("Output destroyed, named: {}", output_name);
		comfy_kernel.output_data_map.remove(&output_name);
	}
}

// ? Handles addition and removal of outputs
pub struct OutputManagerHandler;
impl WLROutputManagerHandler for OutputManagerHandler {
	/// Called whenever an output is added.
	#[wlroots_dehandle(compositor)]
	fn output_added<'output>(
		&mut self,
		compositor_handle: WLRCompositorHandle,
		output_builder: WLROutputBuilder<'output>,
	) -> Option<WLROutputBuilderResult<'output>> {
		let result = output_builder.build_best_mode(OutputHandler);
		use compositor_handle as compositor;
		let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
		comfy_kernel.add_output(&result.output, true);
		Some(result)
	}
}
