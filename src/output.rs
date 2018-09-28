use wlroots::utils::current_time;
use wlroots::{
	project_box as wlr_project_box, Area, CompositorHandle as WLRCompositorHandle, Origin,
	OutputBuilder as WLROutputBuilder, OutputBuilderResult as WLROutputBuilderResult, OutputHandle as WLROutputHandle,
	OutputHandler as WLROutputHandler, OutputLayoutHandler as WLROutputLayoutHandler,
	OutputManagerHandler as WLROutputManagerHandler, Renderer, Size,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

use state::State;

/*
..####...##..##..######..#####...##..##..######.
.##..##..##..##....##....##..##..##..##....##...
.##..##..##..##....##....#####...##..##....##...
.##..##..##..##....##....##......##..##....##...
..####....####.....##....##.......####.....##...
................................................
*/

pub struct OutputLayout;
impl WLROutputLayoutHandler for OutputLayout {}

pub struct Output;
impl WLROutputHandler for Output {
	fn on_frame(&mut self, compositor: WLRCompositorHandle, output: WLROutputHandle) {
		dehandle!(
			@compositor = {compositor};
			@output = {output};
			let state: &mut State = compositor.data.downcast_mut().unwrap();
			let renderer = compositor.renderer
				.as_mut()
				.expect("Compositor was not loaded with a renderer");
			let mut render_context = renderer.render(output, None);
			render_context.clear([135.0/255.0, 7.0/255.0, 52.0/255.0, 1.0]);
			render_shells(state, &mut render_context)
		);
	}
}

pub struct OutputManager;
impl WLROutputManagerHandler for OutputManager {
	fn output_added<'output>(
		&mut self,
		compositor: WLRCompositorHandle,
		builder: WLROutputBuilder<'output>,
	) -> Option<WLROutputBuilderResult<'output>> {
		let mut result = builder.build_best_mode(Output);
		dehandle!(
				@compositor = {compositor};
				@output = {&mut result.output};
				let state: &mut State = compositor.data.downcast_mut().unwrap();
				let xcursor_manager = &mut state.xcursor_manager;
				// TODO use output config if present instead of auto
				let layout = &mut state.layout;
				let cursor = &mut state.cursor;
				@layout = {layout};
				@cursor = {cursor};
				layout.add_auto(output);
				cursor.attach_output_layout(layout);
				xcursor_manager.load(output.scale());
				xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
				let (x, y) = cursor.coords();
				// https://en.wikipedia.org/wiki/Mouse_warping
				cursor.warp(None, x, y)
		);
		Some(result)
	}
}

/// Render the shells in the current compositor state on the given output
/// renderer.
fn render_shells(state: &mut State, renderer: &mut Renderer) {
	let shells: Vec<WLRXdgV6ShellSurfaceHandle> = state.shells.clone();
	for mut shell in shells {
		dehandle!(
			@shell = {shell};
			@surface = {shell.surface()};
			state.current_rotation = state.current_rotation + 0.01;
			@layout = {&state.layout};
			let (width, height) = surface.current_state().size();
			let (render_width, render_height) =
				(
					width * renderer.output.scale() as i32,
					height * renderer.output.scale() as i32
				);
			let (lx, ly) = (0.0, 0.0);
			let render_box = Area::new(
				Origin::new(lx as i32, ly as i32),
				Size::new(render_width,render_height)
			);

			if layout.intersects(renderer.output, render_box) {
				let transform = renderer.output.get_transform().invert();
				let matrix = wlr_project_box(
					render_box,
					transform,
					state.current_rotation,
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
