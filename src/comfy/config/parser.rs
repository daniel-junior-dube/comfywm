use input::keyboard::XkbKeySet;

pub fn convert_to_xkb_string(modkey_str: &str, keyset: &str) -> Result<Vec<String>, String> {
    let splitted_keyset: Vec<&str> = keyset.split("+").map(|key| {
        if key == "$mod" {
            return modkey_str;
        } else {
            return key;
        }
    }).collect();

    let mut cannonicalized_keys: Vec<Vec<String>> = Vec::new();

    for key in splitted_keyset.iter() {
        cannonicalized_keys.push(cannonicalize_key(key)?)
    }

    Ok(create_combinations_as_string(cannonicalized_keys))
}

fn create_combinations_as_string(keys: Vec<Vec<String>>) -> Vec<String> {
    let mut combinations: Vec<String> = Vec::new();
    let mut first_keys = keys;
    let key_remainder = first_keys.split_off(1);

    let mut first_keys = first_keys.pop().unwrap();
    let mut initial_keys: Vec<String> = Vec::new();

    initial_keys.push(first_keys.pop().unwrap());
    if first_keys.len() == 1 {
        initial_keys.push(first_keys.pop().unwrap());
    }

    if key_remainder.len() == 0 {
        return initial_keys;
    } else {
        let remainder_key_combinations: Vec<String> = create_combinations_as_string(key_remainder);
        for inital_key in initial_keys.iter() {
            for key in remainder_key_combinations.iter() {
                let mut key_combination: Vec<String> = Vec::new();
                key_combination.push(inital_key.clone());
                key_combination.push((*key).clone());
                combinations.push(key_combination.join("+"));
            }
        }
    }

    combinations
}

fn cannonicalize_key(key: &str) -> Result<Vec<String>, String> {
    let mut keys: Vec<String> = Vec::new();
    match key {
        "Control" => {
            keys.push("Control_L".to_string());
            keys.push("Control_R".to_string());
        },
        "Alt" => {
            keys.push("Alt_L".to_string());
            keys.push("Alt_R".to_string());
        },
        "Shift" => {
            keys.push("Shift_L".to_string());
            keys.push("Shift_R".to_string());
        },
        _ => {
            XkbKeySet::from_str(key)?;
            keys.push(key.to_string());
        }
    };
    Ok(keys)
}