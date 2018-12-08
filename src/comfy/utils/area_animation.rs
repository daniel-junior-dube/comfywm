use std::time::{Duration, Instant};
use wlroots::{Area, Origin, Size};
use utils::time::duration_to_millis;
use utils::animation::Animation;

/// Area transition animation from a source to a destination. The animation is controlled using a duration in seconds and an animation variant (linear, easing, etc).
/// The animation starts as soon as the object is created. The precision of the animation is handled by polling the animation object more frequently.
#[derive(Clone)]
pub struct AreaAnimation {
	source: Area,
	destination: Area,
	area_diff: Area,
	start_time: Instant,
	duration: Duration,
	transition: Animation
}
impl AreaAnimation {
	pub fn new(source: Area, destination: Area, duration_in_seconds: u8, transition: Animation) -> Self {
		let area_diff = Area::new(
			Origin::new(destination.origin.x - source.origin.x, destination.origin.y - source.origin.y),
			Size::new(destination.size.width - source.size.width, destination.size.height - source.size.height)
		);
		AreaAnimation {
			source,
			destination,
			area_diff,
			start_time: Instant::now(),
			duration: Duration::new(0, duration_in_seconds as u32 * 1000000),
			transition,
		}
	}

	/// Returns the current ratio of the animation progress (between 0.0 and 1.0).
	fn get_current_progress_ratio(&self) -> f32 {
		let current_progress_ratio =
			duration_to_millis(&self.start_time.elapsed()) as f64 /
			duration_to_millis(&self.duration) as f64;
		if current_progress_ratio > 1.0 {
			1.0
		} else {
			current_progress_ratio as f32
		}
	}

	/// Returns the area associated with the animation's progress.
	pub fn current_area(&mut self) -> Area {
		if self.has_ended() {
			self.destination.clone()
		} else {
			let transition_progress = self.transition.calculate(
				self.get_current_progress_ratio()
			);
			Area::new(
				Origin::new(
					self.source.origin.x + (self.area_diff.origin.x as f32 * transition_progress) as i32,
					self.source.origin.y + (self.area_diff.origin.y as f32 * transition_progress) as i32
				),
				Size::new(
					self.source.size.width + (self.area_diff.size.width as f32 * transition_progress) as i32,
					self.source.size.height + (self.area_diff.size.height as f32 * transition_progress) as i32
				)
			)
		}
	}

	/// Returns `true` if the animation has ended.
	pub fn has_ended(&self) -> bool {
		self.start_time.elapsed() >= self.duration
	}
}