// ANCHOR: code
#[derive(Default)]
pub struct Wallet {
    balance: u64,
}

impl Wallet {
    pub fn deposit(&mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }
}
// ANCHOR_END: code

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposits_into_the_wallet() {
        let mut wallet = Wallet::default();

        wallet.deposit(10);

        let got = wallet.balance();
        let want = 10;

        assert_eq!(got, want);
    }
}
// ANCHOR_END: test
