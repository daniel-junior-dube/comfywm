use std::time::Duration;

/// Conversion a duration object to milliseconds
pub fn duration_to_millis(duration: &Duration) -> u64 {
	let nanos = duration.subsec_nanos() as u64;
	(1000 * 1000 * 1000 * duration.as_secs() + nanos) / (1000 * 1000)
}