/// Totals a list of amounts with the ledger's own addition.
pub fn total(amounts: &[i64]) -> i64 {
    amounts
        .iter()
        .fold(0, |sum, amount| ledger::add(sum, *amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_a_list_of_amounts() {
        assert_eq!(total(&[1, 2, 3]), 6);
    }
}
