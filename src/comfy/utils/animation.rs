/// Enum of all animation variants.
#[derive(Clone)]
#[allow(dead_code)]
pub enum Animation {
	Linear,
	EaseInQuad,
	EaseOutQuad,
	EaseInOutQuad,
	EaseInCubic,
	EaseOutCubic,
	EaseInOutCubic,
	EaseInQuart,
	EaseOutQuart,
	EaseInOutQuart,
	EaseInQuint,
	EaseOutQuint,
	EaseInOutQuint,
	EaseInCirc,
	EaseOutCirc,
	EaseInOutCirc,
}
impl Animation {
	pub fn calculate(&self, t: f32) -> f32 {
		match self {
			Animation::Linear => ease::linear(t),
			Animation::EaseInQuad => ease::ease_in_quad(t),
			Animation::EaseOutQuad => ease::ease_out_quad(t),
			Animation::EaseInOutQuad => ease::ease_in_out_quad(t),
			Animation::EaseInCubic => ease::ease_in_cubic(t),
			Animation::EaseOutCubic => ease::ease_out_cubic(t),
			Animation::EaseInOutCubic => ease::ease_in_out_cubic(t),
			Animation::EaseInQuart => ease::ease_in_quart(t),
			Animation::EaseOutQuart => ease::ease_out_quart(t),
			Animation::EaseInOutQuart => ease::ease_in_out_quart(t),
			Animation::EaseInQuint => ease::ease_in_quint(t),
			Animation::EaseOutQuint => ease::ease_out_quint(t),
			Animation::EaseInOutQuint => ease::ease_in_out_quint(t),
			Animation::EaseInCirc => ease::ease_in_circ(t),
			Animation::EaseOutCirc => ease::ease_out_circ(t),
			Animation::EaseInOutCirc => ease::ease_in_out_circ(t),
		}
	}
}

/// Easing animation module. Mainly converted from the c++ implementation of http://robertpenner.com/easing/.
/// These functions where originally created by Robert Penner.
pub mod ease {
	// no easing, no acceleration
	pub fn linear(t: f32) -> f32 {
		t
	}

	// accelerating from zero velocity
	pub fn ease_in_quad(t: f32) -> f32 {
		t * t
	}

	// decelerating to zero velocity
	pub fn ease_out_quad(t: f32) -> f32 {
		t * (2.0 - t)
	}

	// acceleration until halfway, then deceleration
	pub fn ease_in_out_quad(t: f32) -> f32 {
		if t < 0.5 {
			2.0 * t * t
		} else {
			-1.0 + (4.0 - 2.0 * t) * t
		}
	}

	// accelerating from zero velocity
	pub fn ease_in_cubic(t: f32) -> f32 {
		t * t * t
	}

	// decelerating to zero velocity
	pub fn ease_out_cubic(t: f32) -> f32 {
		let t2 = t - 1.0;
		t2 * t2 * t2 + 1.0
	}

	// acceleration until halfway, then deceleration
	pub fn ease_in_out_cubic(t: f32) -> f32 {
		if t < 0.5 {
			4.0 * t * t * t
		} else {
			(t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
		}
	}

	// accelerating from zero velocity
	pub fn ease_in_quart(t: f32) -> f32 {
		t * t * t * t
	}

	// decelerating to zero velocity
	pub fn ease_out_quart(t: f32) -> f32 {
		let t2 = t - 1.0;
		1.0 - t2 * t2 * t2 * t2
	}

	// acceleration until halfway, then deceleration
	pub fn ease_in_out_quart(t: f32) -> f32 {
		if t < 0.5 {
			8.0 * t * t * t * t
		} else {
			let t2 = t - 1.0;
			1.0 - 8.0 * t2 * t2 * t2 * t2
		}
	}

	// accelerating from zero velocity
	pub fn ease_in_quint(t: f32) -> f32 {
		t * t * t * t * t
	}

	// decelerating to zero velocity
	pub fn ease_out_quint(t: f32) -> f32 {
		let t2 = t - 1.0;
		1.0 + t2 * t2 * t2 * t2 * t2
	}

	// acceleration until halfway, then deceleration
	pub fn ease_in_out_quint(t: f32) -> f32 {
		if t < 0.5 {
			16.0 * t * t * t * t * t
		} else {
			let t2 = t - 1.0;
			1.0 + 16.0 * t2 * t2 * t2 * t2 * t2
		}
	}

	// accelerating from zero velocity
	pub fn ease_in_circ(t: f32) -> f32 {
		1.0 - (1.0 - t * t).sqrt()
	}

	// decelerating to zero velocity
	pub fn ease_out_circ(t: f32) -> f32 {
		((2.0 - t) * t).sqrt()
	}

	// acceleration until halfway, then deceleration
	pub fn ease_in_out_circ(t: f32) -> f32 {
		if t < 0.5 {
			0.5 * (1.0 - (1.0 - 4.0 * t * t).sqrt())
		} else {
			0.5 * ((-(2.0 * t - 3.0) * (2.0 * t - 1.0)).sqrt() + 1.0)
		}
	}
}
