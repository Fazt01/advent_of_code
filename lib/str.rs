pub trait StrExt {
    fn char_at(&self, i: usize) -> char;
}

impl StrExt for str {
    fn char_at(&self, i: usize) -> char {
        self[i..].chars().next().unwrap()
    }
}