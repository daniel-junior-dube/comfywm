use wlroots::utils::current_time;
use wlroots::{
	project_box as wlr_project_box, Area, CompositorHandle as WLRCompositorHandle, Origin,
	OutputBuilder as WLROutputBuilder, OutputBuilderResult as WLROutputBuilderResult, OutputHandle as WLROutputHandle,
	OutputHandler as WLROutputHandler,
	OutputLayoutHandler as WLROutputLayoutHandler, /* , OutputDestruction as WLROutputDestruction */
	OutputManagerHandler as WLROutputManagerHandler, Renderer, Size,
};

use common::colors::Color;
use compositor::window::Window;
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
impl OutputHandler {
	/// Renders the provided window data using the provided renderer.
	fn render_window(&self, window: &Window, renderer: &mut Renderer) {
		let Window {
			shell_handle,
			area: window_area,
		} = window;
		with_handles!([(shell: {shell_handle}), (surface: {shell.surface()})] => {
			let (width, height) = surface.current_state().size();
			let (render_width, render_height) = (
				width * renderer.output.scale() as i32,
				height * renderer.output.scale() as i32
			);
			let render_box = Area::new(
				Origin::new(window_area.origin.x, window_area.origin.y),
				Size::new(render_width, render_height)
			);
			let transform = renderer.output.get_transform().invert();
			let matrix = wlr_project_box(
				*window_area,
				transform,
				0.0,
				renderer.output.transform_matrix()
			);
			if let Some(texture) = surface.texture().as_ref() {
				renderer.render_texture_with_matrix(texture, matrix);
			}
			surface.send_frame_done(current_time());
		}).unwrap();
	}
}
impl WLROutputHandler for OutputHandler {
	/// Called every time the output frame is updated.
	fn on_frame(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		with_handles!([(compositor: {compositor_handle}), (output: {output_handle})] => {
			let output_name = output.name().clone();
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			let renderer = compositor.renderer
				.as_mut()
				.expect("Compositor was not loaded with a renderer");
			let mut render_context = renderer.render(output, None);

			// ? Clearing the screen and get indices of windows to render
			let mut render_color = Color::black().as_rgba_slice();
			let mut windows = vec![];
			if let Some(OutputData {workspace, clear_color, ..}) = comfy_kernel.output_data_map.get(&output_name) {
				render_color = *clear_color;
				windows = workspace.window_layout.get_windows();
			}

			// ? Clear the screen with the render color
			render_context.clear(render_color);

			// ? Render each window
			windows.iter().for_each(|window_ref| self.render_window(window_ref, &mut render_context));
		}).unwrap()
	}

	/// WIP
	/// Called every time the output frame is updated.
	fn on_transform(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		with_handles!([(compositor: {compositor_handle}), (output: {output_handle})] => {
			let output_name = output.name().clone();
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			if let Some(OutputData {workspace, ..}) = comfy_kernel.output_data_map.get(&output_name) {
				let mut output_layout_handle = &comfy_kernel.output_layout_handle;
				with_handles!([(output_layout: {output_layout_handle})] => {
					// TODO: If window_layout area doesn't intersect the output, refresh the layout
					let render_box = workspace.window_layout.area().unwrap();
					let output_layout_box = output_layout.get_box(None);
				}).unwrap();
			}
		}).unwrap()
	}

	/// Called every time the output mode changes.
	fn on_mode_change(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		dehandle!(
			@compositor = {compositor_handle};
			@output = {output_handle};
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			let output_data_map = &mut comfy_kernel.output_data_map;
			if let Some(output_data) = output_data_map.get_mut(&output.name()) {
				let (x, y) = output.layout_space_pos();
				let (width, height) = output.effective_resolution();
				let updated_windows_with_area = output_data.workspace.window_layout.update_area_and_rebalance(
					Area::new(
						Origin::new(x, y),
						Size::new(width, height)
					)
				);
				// TODO: IMPORTANT! - handle updated_windows_with_area
			}
		);
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
	fn destroyed(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		dehandle!(
			@compositor = {compositor_handle};
			@output = {output_handle};
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			let output_name = output.name();
			println!("Output destroyed, named: {}", output_name);
			// TODO: Transfer data from destroyed output data to main output data
			/*{
				let destroyed_output_data = comfy_kernel.output_data_map.get(&output_name);
				let main_output_data
			} */
			comfy_kernel.output_data_map.remove(&output_name);
			println!("Removed OutputData in from data_map! Nb of total entries: {}", comfy_kernel.output_data_map.len());
			()
		);
	}
}

// ? Handles addition and removal of outputs
pub struct OutputManagerHandler;
impl WLROutputManagerHandler for OutputManagerHandler {
	/// Called whenever an output is added.
	fn output_added<'output>(
		&mut self,
		compositor_handle: WLRCompositorHandle,
		output_builder: WLROutputBuilder<'output>,
	) -> Option<WLROutputBuilderResult<'output>> {
		let mut result = output_builder.build_best_mode(OutputHandler);
		dehandle!(
			@compositor = {compositor_handle};
			@output = {&mut result.output};
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			comfy_kernel.active_output_name = output.name();
			let xcursor_manager = &mut comfy_kernel.xcursor_manager;
			// TODO use output config if present instead of auto
			let output_layout_handle = &mut comfy_kernel.output_layout_handle;
			let cursor_handle = &mut comfy_kernel.cursor_handle;
			let output_data_map = &mut comfy_kernel.output_data_map;

			@output_layout = {output_layout_handle};
			@cursor = {cursor_handle};

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
				OutputData::new(
					Area::new(
						Origin::new(x, y),
						Size::new(width, height)
					)
				)
			);
			()
		);
		Some(result)
	}
}
