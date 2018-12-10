use wlroots::{
	Area, CompositorHandle as WLRCompositorHandle, Origin, OutputBuilder as WLROutputBuilder,
	OutputBuilderResult as WLROutputBuilderResult, OutputHandle as WLROutputHandle, OutputHandler as WLROutputHandler,
	OutputLayoutHandler as WLROutputLayoutHandler, /* , OutputDestruction as WLROutputDestruction */
	OutputManagerHandler as WLROutputManagerHandler, Size,
};

use common::colors::Color;
use compositor::workspace::Workspace;
use compositor::ComfyKernel;

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
