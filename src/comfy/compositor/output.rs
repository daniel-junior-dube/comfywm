use wlroots::utils::current_time;
use wlroots::{
	project_box as wlr_project_box, Area, CompositorHandle as WLRCompositorHandle, Origin,
	OutputBuilder as WLROutputBuilder, OutputBuilderResult as WLROutputBuilderResult, OutputHandle as WLROutputHandle,
	OutputHandler as WLROutputHandler,
	OutputLayoutHandler as WLROutputLayoutHandler, /* , OutputDestruction as WLROutputDestruction */
	OutputManagerHandler as WLROutputManagerHandler, Renderer, Size, SurfaceHandle as WLRSurfaceHandle,
};

use common::colors::Color;
use compositor::window::Window;
use compositor::workspace::Workspace;
use compositor::ComfyKernel;
use utils::handle_helper;

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
impl OutputHandler {
	/// Renders the provided window data using the provided renderer.
	fn render_window(&self, window: &Window, renderer: &mut Renderer) {
		let window_area = window.area.clone();
		window.for_each_surface(&mut |surface_handle: WLRSurfaceHandle, sx, sy| {
			self.render_surface(renderer, &surface_handle, &window_area, sx, sy);
		});
	}

	/// Renders the provided surface using the provided renderer.
	#[wlroots_dehandle(surface)]
	fn render_surface(
		&self,
		renderer: &mut Renderer,
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
}
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
		let mut render_color = Color::black().as_rgba_slice();
		let mut windows = vec![];
		if let Some(OutputData {
			workspace, clear_color, ..
		}) = comfy_kernel.output_data_map.get(&output_name)
		{
			render_color = *clear_color;
			windows = workspace.window_layout.get_windows();
		}

		// ? Clear the screen with an image or the render color otherwise
		if let Some(wallpaper_texture) = &comfy_kernel.wallpaper_texture {
			let (texture_width, texture_height) = wallpaper_texture.size();
			let (scale_x, scale_y) = (
				output_width as f32 / texture_width as f32,
				output_height as f32 / texture_height as f32,
			);
			transform_matrix[0] = transform_matrix[0] * scale_x;
			transform_matrix[4] = transform_matrix[4] * scale_y;
			render_context.render_texture(&wallpaper_texture, transform_matrix, 0, 0, 1.0);
		} else {
			render_context.clear(render_color);
		}

		// ? Render each window
		windows
			.iter()
			.for_each(|window_ref| self.render_window(window_ref, &mut render_context));
	}

	/// WIP
	/// Called every time the output frame is updated.
	fn on_transform(&mut self, _: WLRCompositorHandle, _: WLROutputHandle) {
		//println!("on_transform");
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
		//println!("on_enable");
	}

	/// Called every time the output scale changes.
	fn on_scale_change(&mut self, _: WLRCompositorHandle, _: WLROutputHandle) {
		//println!("on_scale_change");
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
		debug!(
			"Removed OutputData from data_map! Nb of total entries: {}",
			comfy_kernel.output_data_map.len()
		);
		()
	}
}

// ? Handles addition and removal of outputs
pub struct OutputManagerHandler;
impl WLROutputManagerHandler for OutputManagerHandler {
	/// Called whenever an output is added.
	#[wlroots_dehandle(compositor, output, output_layout, cursor)]
	fn output_added<'output>(
		&mut self,
		compositor_handle: WLRCompositorHandle,
		output_builder: WLROutputBuilder<'output>,
	) -> Option<WLROutputBuilderResult<'output>> {
		let result = output_builder.build_best_mode(OutputHandler);
		{
			let output_handle = &result.output;
			use compositor_handle as compositor;
			use output_handle as output;

			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			comfy_kernel.active_output_name = output.name();
			let xcursor_manager = &mut comfy_kernel.xcursor_manager;
			// TODO use output config if present instead of auto
			let output_layout_handle = &mut comfy_kernel.output_layout_handle;
			let cursor_handle = &mut comfy_kernel.cursor_handle;
			let output_data_map = &mut comfy_kernel.output_data_map;

			use cursor_handle as cursor;
			use output_layout_handle as output_layout;

			output_layout.add_auto(output);
			cursor.attach_output_layout(output_layout);
			xcursor_manager.load(output.scale());
			xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
			let (x, y) = cursor.coords();
			// https://en.wikipedia.org/wiki/Mouse_warping
			cursor.warp(None, x, y);

			println!("New output detected, named: {}", output.name());
			let (x, y) = output.layout_space_pos();
			let (width, height) = output.effective_resolution();
			output_data_map.insert(
				output.name(),
				OutputData::new(Area::new(Origin::new(x, y), Size::new(width, height))),
			);
		}
		Some(result)
	}
}
