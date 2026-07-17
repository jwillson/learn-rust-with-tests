use std::ops::AddAssign;

// ANCHOR: bitcoin
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Bitcoin(pub u64);

impl AddAssign for Bitcoin {
    fn add_assign(&mut self, rhs: Bitcoin) {
        self.0 += rhs.0;
    }
}
// ANCHOR_END: bitcoin

// ANCHOR: wallet
#[derive(Default)]
pub struct Wallet {
    balance: Bitcoin,
}

impl Wallet {
    pub fn deposit(&mut self, amount: Bitcoin) {
        self.balance += amount;
    }

    pub fn balance(&self) -> Bitcoin {
        self.balance
    }
}
// ANCHOR_END: wallet

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn deposits_into_the_wallet() {
        let mut wallet = Wallet::default();

        wallet.deposit(Bitcoin(10));

        let got = wallet.balance();
        let want = Bitcoin(10);

        assert_eq!(got, want);
    }
    // ANCHOR_END: test
}
