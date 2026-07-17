#[derive(Debug)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub sum: f64,
}

// ANCHOR: code
pub fn balance_for(transactions: &[Transaction], name: &str) -> f64 {
    transactions.iter().fold(0.0, |balance, transaction| {
        if transaction.from == name {
            balance - transaction.sum
        } else if transaction.to == name {
            balance + transaction.sum
        } else {
            balance
        }
    })
}
// ANCHOR_END: code

// ANCHOR: total_sent
pub fn total_sent_by(transactions: &[Transaction], name: &str) -> f64 {
    transactions
        .iter()
        .filter(|transaction| transaction.from == name)
        .map(|transaction| transaction.sum)
        .sum()
}
// ANCHOR_END: total_sent

// ANCHOR: find
pub fn first_payment_to<'a>(
    transactions: &'a [Transaction],
    name: &str,
) -> Option<&'a Transaction> {
    transactions
        .iter()
        .find(|transaction| transaction.to == name)
}
// ANCHOR_END: find

#[cfg(test)]
mod tests {
    use super::*;

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

    // ANCHOR: extra_tests
    #[test]
    fn totals_only_the_money_a_person_sent() {
        let transactions = transactions();

        assert_eq!(total_sent_by(&transactions, "Chris"), 100.0);
        assert_eq!(total_sent_by(&transactions, "Riya"), 0.0);
    }

    #[test]
    fn finds_the_first_payment_to_a_person() {
        let transactions = transactions();

        let payment = first_payment_to(&transactions, "Chris");
        assert_eq!(payment.expect("expected a payment").sum, 25.0);

        assert!(first_payment_to(&transactions, "Nobody").is_none());
    }
    // ANCHOR_END: extra_tests
}
