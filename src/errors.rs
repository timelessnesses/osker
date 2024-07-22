#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errors {
    RankNotFoundError,
}

impl std::error::Error for Errors {}
impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:#?}", self))
    }
}
