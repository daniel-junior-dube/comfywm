use wlroots::utils::current_time;
use wlroots::{
	project_box as wlr_project_box, Area, CompositorHandle as WLRCompositorHandle, Origin,
	OutputBuilder as WLROutputBuilder, OutputBuilderResult as WLROutputBuilderResult, OutputHandle as WLROutputHandle,
	OutputHandler as WLROutputHandler, OutputLayoutHandler as WLROutputLayoutHandler,
	OutputManagerHandler as WLROutputManagerHandler, Renderer, Size,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

use compositor::ComfyKernel;

/*
..####...##..##..######..#####...##..##..######.
.##..##..##..##....##....##..##..##..##....##...
.##..##..##..##....##....#####...##..##....##...
.##..##..##..##....##....##......##..##....##...
..####....####.....##....##.......####.....##...
................................................
*/

// ? Handles events on the output layout (how displays are organized)
pub struct OutputLayoutHandler;
impl WLROutputLayoutHandler for OutputLayoutHandler {}

// ? Handles the actions and events of a specific output
pub struct OutputHandler;
impl OutputHandler {
	/// Render the shells in the current compositor kernel on the given output
	/// renderer.
	fn render_shells(&mut self, comfy_kernel: &mut ComfyKernel, renderer: &mut Renderer) {
		let shells: Vec<WLRXdgV6ShellSurfaceHandle> = comfy_kernel.shells.clone();
		for shell in shells {
			self.render_shell(shell, comfy_kernel, renderer);
		}
	}

	/// Render a given shell on the given output renderer.
	fn render_shell(
		&mut self,
		shell: WLRXdgV6ShellSurfaceHandle,
		comfy_kernel: &mut ComfyKernel,
		renderer: &mut Renderer,
	) {
		let x = comfy_kernel.x;
		let y = comfy_kernel.y;
		dehandle!(
			@shell = {shell};
			@surface = {shell.surface()};
			@layout = {&comfy_kernel.output_layout_handle};
			let (width, height) = surface.current_state().size();
			let (render_width, render_height) = (
				width * renderer.output.scale() as i32,
				height * renderer.output.scale() as i32
			);
			let render_box = Area::new(
				Origin::new(x, y),
				Size::new(render_width,render_height)
			);
			if layout.intersects(renderer.output, render_box) {
				let transform = renderer.output.get_transform().invert();
				let matrix = wlr_project_box(
					render_box,
					transform,
					0.0,
					renderer.output.transform_matrix()
				);
				if let Some(texture) = surface.texture().as_ref() {
					renderer.render_texture_with_matrix(texture, matrix);
				}
				surface.send_frame_done(current_time());
			};
			()
		);
	}
}
impl WLROutputHandler for OutputHandler {
	fn on_frame(&mut self, compositor: WLRCompositorHandle, output: WLROutputHandle) {
		dehandle!(
			@compositor = {compositor};
			@output = {output};
			let comfy_kernel: &mut ComfyKernel = compositor.data.downcast_mut().unwrap();
			let renderer = compositor.renderer
				.as_mut()
				.expect("Compositor was not loaded with a renderer");
			let mut render_context = renderer.render(output, None);
			render_context.clear([135.0/255.0, 7.0/255.0, 52.0/255.0, 1.0]);
			self.render_shells(comfy_kernel, &mut render_context)
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
			let xcursor_manager = &mut comfy_kernel.xcursor_manager;
			// TODO use output config if present instead of auto
			let output_layout_handle = &mut comfy_kernel.output_layout_handle;
			let cursor_handle = &mut comfy_kernel.cursor_handle;
			@output_layout = {output_layout_handle};
			@cursor = {cursor_handle};
			output_layout.add_auto(output);
			cursor.attach_output_layout(output_layout);
			xcursor_manager.load(output.scale());
			xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
			let (x, y) = cursor.coords();
			// https://en.wikipedia.org/wiki/Mouse_warping
			cursor.warp(None, x, y)
		);
		Some(result)
	}
}
