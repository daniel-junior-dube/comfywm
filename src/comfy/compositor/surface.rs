use wlroots::{
	CompositorHandle as WLRCompositorHandle, SurfaceHandle as WLRSurfaceHandle, SurfaceHandler as WLRSurfaceHandler,
};

/*
..####...##..##..#####...######...####....####...######.
.##......##..##..##..##..##......##..##..##..##..##.....
..####...##..##..#####...####....######..##......####...
.....##..##..##..##..##..##......##..##..##..##..##.....
..####....####...##..##..##......##..##...####...######.
........................................................
*/

pub struct SurfaceHandler;
impl WLRSurfaceHandler for SurfaceHandler {
	// ? A surface commit is done when changes made to a surface a completed and ready to be applied
	fn on_commit(&mut self, _: WLRCompositorHandle, surface: WLRSurfaceHandle) {
		println!("Commiting for surface {:?}", surface);
	}
}
