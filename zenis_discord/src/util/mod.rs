mod emoji;
mod extensions;

pub use emoji::*;
pub use extensions::*;

pub fn bold(string: &str) -> String {
    format!("**{}**", string)
}

pub fn simple_markdown(string: &str) -> String {
    format!("```{string}```")
}

pub fn code_markdown(code_lang: &str, string: &str) -> String {
    format!("```{code_lang}\n{string}\n```")
}
