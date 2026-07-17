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

impl fmt::Display for Bitcoin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} BTC", self.0)
    }
}

// ANCHOR: error
#[derive(Debug, PartialEq, Eq)]
pub struct InsufficientFundsError;
// ANCHOR_END: error

#[derive(Default)]
pub struct Wallet {
    pub balance: Bitcoin,
}

// ANCHOR: code
impl Wallet {
    pub fn deposit(&mut self, amount: Bitcoin) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: Bitcoin) -> Result<(), InsufficientFundsError> {
        if amount > self.balance {
            return Err(InsufficientFundsError);
        }

        self.balance -= amount;
        Ok(())
    }

    pub fn balance(&self) -> Bitcoin {
        self.balance
    }
}
// ANCHOR_END: code

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

    // ANCHOR: test
    #[test]
    fn withdraws_from_the_wallet() {
        let mut wallet = Wallet {
            balance: Bitcoin(20),
        };

        let result = wallet.withdraw(Bitcoin(10));

        assert!(result.is_ok());
        assert_balance(&wallet, Bitcoin(10));
    }

    #[test]
    fn refuses_to_withdraw_more_than_the_balance() {
        let mut wallet = Wallet {
            balance: Bitcoin(20),
        };

        let result = wallet.withdraw(Bitcoin(100));

        assert_eq!(result, Err(InsufficientFundsError));
        assert_balance(&wallet, Bitcoin(20));
    }
    // ANCHOR_END: test
}
