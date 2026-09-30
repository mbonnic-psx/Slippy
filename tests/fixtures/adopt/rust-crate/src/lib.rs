/// Adds two amounts.
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_two_amounts() {
        assert_eq!(add(2, 3), 5);
    }
}
