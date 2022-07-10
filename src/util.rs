pub fn str_to_char_arr(s: &str) -> Vec<i32> {
    let mut output = Vec::with_capacity(s.len());
    for c in s.chars() {
        output.push(c as u8 as i32);
    }
    output
}

pub fn char_arr_to_str(arr: &Vec<i32>) -> String {
    let mut output = String::new();
    for c in arr {
        output.push((*c) as u8 as char);
    }
    output
}
