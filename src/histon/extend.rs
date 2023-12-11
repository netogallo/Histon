use super::foundations::*;

pub struct Extend<TRelIn, F> {
    relation : TRelIn,
    columns : Vec<String>,
    f : F
}