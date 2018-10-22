use input::keyboard::XkbKeySet;

/// Convert a keyset in Comfy's format and convert it to a valid XkbKeySet
pub fn convert_to_xkb_string(modkey_str: &str, keyset: &str) -> Result<Vec<String>, String> {
	// Replace all the $mod in the file with the value of modkey
	let splitted_keyset: Vec<&str> = keyset
		.split("+")
		.map(|key|
			if key == "$mod" {
				modkey_str
			} else {
				key
			}
		).collect();

	let mut cannonicalized_keys: Vec<Vec<String>> = Vec::new();
	for key in splitted_keyset.iter() {
		cannonicalized_keys.push(canonicalize_key(key)?)
	}

	// Return all the possible combinations of valid XkbKeySet as a multiple Strings
	Ok(create_combinations_as_string(cannonicalized_keys))
}

/// A recursive function that take multiple keys and return all the possible combinations within them.
///
/// #Example
/// The following value : `[["Control_L", "Control_R"], ["Shift_L", "Shift_R"], ["Up"]]`
/// returns : ```["Control_R+Shift_R+Up",
///               "Control_R+Shift_L+Up",
///               "Control_L+Shift_R+Up",
///               "Control_L+Shift_L+Up"]```
fn create_combinations_as_string(keys: Vec<Vec<String>>) -> Vec<String> {
	// Split the keys in two groups
	// Ex: [["Control_L", "Control_R"], ["Shift_L", "Shift_R"], ["Up"]] =>
	// key_head_group = ["Control_L", "Control_R"]
	// key_tail_group = [["Shift_L", "Shift_R"], ["Up"]]
	let mut key_head_group = keys;
	let key_tail_group = key_head_group.split_off(1);

	// Take the key_head_group and flatten it into a vector.
	let prefix_keys: Vec<String> = key_head_group.into_iter().flatten().collect();

	// Base case: return the prefix if the remainder is empty.
	if key_tail_group.len() == 0 {
		prefix_keys
	} else {
		let mut combinations: Vec<String> = Vec::new();

		// Recursive step: call the function it self with the remainder until it is empty.
		let remainder_key_combinations: Vec<String> = create_combinations_as_string(key_tail_group);

		// Create the combinations themselves with all the prefix key and the recursive calls.
		for inital_key in prefix_keys.iter() {
			for key in remainder_key_combinations.iter() {
				combinations.push(vec![inital_key.clone(), key.clone()].join("+"));
			}
		}

		combinations
	}
}

/// Convert a Comfy's modifier class into a valid XkbKeySet if it needs to.
fn canonicalize_key(key: &str) -> Result<Vec<String>, String> {
	match key {
		"Control" => Ok(vec!["Control_L".to_string(), "Control_R".to_string()]),
		"Alt" => Ok(vec!["Alt_L".to_string(), "Alt_R".to_string()]),
		"Shift" => Ok(vec!["Shift_L".to_string(), "Shift_R".to_string()]),
		_ => {
			XkbKeySet::from_str(key)?;
			Ok(vec![key.to_string()])
		}
	}
}
