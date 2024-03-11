use serde::Serializer;

pub trait Stringify {
    fn stringify(&self) -> Option<String>;
}

impl Stringify for Option<i64> {
    fn stringify(&self) -> Option<String> {
        self.map(|c| c.to_string())
    }
}

pub fn option_stringify<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Stringify,
    S: Serializer,
{
    serializer.serialize_str(&value.stringify().unwrap())
}
