// ANCHOR: types
#[derive(Debug)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub sum: f64,
}
// ANCHOR_END: types

// ANCHOR: code
pub fn balance_for(transactions: &[Transaction], name: &str) -> f64 {
    let mut balance = 0.0;
    for transaction in transactions {
        if transaction.from == name {
            balance -= transaction.sum;
        }
        if transaction.to == name {
            balance += transaction.sum;
        }
    }
    balance
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    fn transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                from: "Chris".to_string(),
                to: "Riya".to_string(),
                sum: 100.0,
            },
            Transaction {
                from: "Adil".to_string(),
                to: "Chris".to_string(),
                sum: 25.0,
            },
        ]
    }

    #[test]
    fn balances_reflect_money_in_and_money_out() {
        let transactions = transactions();

        assert_eq!(balance_for(&transactions, "Riya"), 100.0);
        assert_eq!(balance_for(&transactions, "Chris"), -75.0);
        assert_eq!(balance_for(&transactions, "Adil"), -25.0);
    }
    // ANCHOR_END: test
}
