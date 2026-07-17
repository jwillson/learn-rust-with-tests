use std::fmt;
use std::ops::{AddAssign, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Bitcoin(pub u64);

impl AddAssign for Bitcoin {
    fn add_assign(&mut self, rhs: Bitcoin) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Bitcoin {
    fn sub_assign(&mut self, rhs: Bitcoin) {
        self.0 -= rhs.0;
    }
}

// ANCHOR: display
impl fmt::Display for Bitcoin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} BTC", self.0)
    }
}
// ANCHOR_END: display

// ANCHOR: wallet
#[derive(Default)]
pub struct Wallet {
    balance: Bitcoin,
}

impl Wallet {
    pub fn deposit(&mut self, amount: Bitcoin) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: Bitcoin) {
        self.balance -= amount;
    }

    pub fn balance(&self) -> Bitcoin {
        self.balance
    }
}
// ANCHOR_END: wallet

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn assert_balance(wallet: &Wallet, want: Bitcoin) {
        let got = wallet.balance();

        assert_eq!(got, want, "got {got} want {want}");
    }

    #[test]
    fn deposits_into_the_wallet() {
        let mut wallet = Wallet::default();

        wallet.deposit(Bitcoin(10));

        assert_balance(&wallet, Bitcoin(10));
    }

    #[test]
    fn withdraws_from_the_wallet() {
        let mut wallet = Wallet {
            balance: Bitcoin(20),
        };

        wallet.withdraw(Bitcoin(10));

        assert_balance(&wallet, Bitcoin(10));
    }
}
// ANCHOR_END: test
