use regex::Regex;

pub fn remove_italic_actions(input: &str) -> String {
    let re = Regex::new(r"[_*][^_*]+[_*]").unwrap();
    let output = re.replace_all(input, "");
    output.trim().to_string()
}
