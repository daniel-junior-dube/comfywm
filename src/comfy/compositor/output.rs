use wlroots::utils::current_time;
use wlroots::{
	project_box as wlr_project_box, Area, CompositorHandle as WLRCompositorHandle, Origin,
	OutputBuilder as WLROutputBuilder, OutputBuilderResult as WLROutputBuilderResult, OutputHandle as WLROutputHandle,
	OutputHandler as WLROutputHandler,
	OutputLayoutHandler as WLROutputLayoutHandler, /* , OutputDestruction as WLROutputDestruction */
	OutputManagerHandler as WLROutputManagerHandler, Renderer, Size,
};

use compositor::workspace::Workspace;
use compositor::ComfyKernel;
use layout::WindowData;

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
}

impl OutputData {
	pub fn new_empty() -> Self {
		OutputData::new(Area::new(Origin::new(0, 0), Size::new(0, 0)))
	}

	pub fn new(area: Area) -> Self {
		OutputData {
			workspace: Workspace::new(area),
		}
	}

	pub fn update_area(&mut self, area: Area) {
		self.workspace.window_layout.update_area(area);
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
	fn render_window(&self, window_data: &WindowData, renderer: &mut Renderer) {
		let WindowData {shell_handle, area} = window_data;
		with_handles!([(shell: {shell_handle}), (surface: {shell.surface()})] => {
			if true {
				let transform = renderer.output.get_transform().invert();
				let matrix = wlr_project_box(
					*area,
					transform,
					0.0,
					renderer.output.transform_matrix()
				);
				if let Some(texture) = surface.texture().as_ref() {
					renderer.render_texture_with_matrix(texture, matrix);
				}
				surface.send_frame_done(current_time());
			};
		}).unwrap();
	}
}
impl WLROutputHandler for OutputHandler {
	fn on_frame(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		with_handles!([(compositor: {compositor_handle}), (output: {output_handle})] => {
			let output_name = output.name().clone();
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			let renderer = compositor.renderer
				.as_mut()
				.expect("Compositor was not loaded with a renderer");
			let mut render_context = renderer.render(output, None);
			render_context.clear([135.0/255.0, 7.0/255.0, 52.0/255.0, 1.0]);

			// ? Render the windows from the workspace's layout
			if let Some(OutputData {workspace, ..}) = comfy_kernel.output_data_map.get(&output_name) {
				for window in workspace.window_layout.windows_data() {
					self.render_window(&window, &mut render_context);
				}
			}
		}).unwrap()
	}

	fn on_transform(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		with_handles!([(compositor: {compositor_handle}), (output: {output_handle})] => {
			let output_name = output.name().clone();
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			if let Some(OutputData {workspace, ..}) = comfy_kernel.output_data_map.get(&output_name) {
				let mut output_layout_handle = &comfy_kernel.output_layout_handle;
				with_handles!([(output_layout: {output_layout_handle})] => {
					// TODO: If window_layout area doesn't intersect the output, refresh the layout
					let render_box = workspace.window_layout.render_box().unwrap();
					let output_layout_box = output_layout.get_box(None);
				}).unwrap();
			}
		}).unwrap()
	}

	/// Called every time the output mode changes.
	fn on_mode_change(&mut self, compositor_handle: WLRCompositorHandle, output_handle: WLROutputHandle) {
		// TODO: Resize output layout with output size
		dehandle!(
			@compositor = {compositor_handle};
			@output = {output_handle};
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			let output_data_map = &mut comfy_kernel.output_data_map;
			let (width, height) = output.size();
			if let Some(output_data) = output_data_map.get_mut(&output.name()) {
				output_data.update_area(
					Area::new(
						Origin::new(0, 0),
						Size::new(width, height)
					)
				);
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
			output_data_map.insert(
				output.name(),
				OutputData::new_empty()
			);
			()
		);
		Some(result)
	}
}
