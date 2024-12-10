use std::str::FromStr;

use super::utils::round_decimal;
use chrono::{Days, NaiveDate};
use rust_decimal::prelude::RoundingStrategy;
use rust_decimal::{Decimal, MathematicalOps};

const INTEREST_SCALE: u32 = 2;
const INTEREST_PRECISION: u32 = 28;
const INTEREST_ROUNDING: RoundingStrategy = RoundingStrategy::MidpointNearestEven;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InterestMethod {
    Convention30_360,
    Actual365,
    Actual360,
    ActualActual,
}

impl FromStr for InterestMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Convention30_360" => Ok(InterestMethod::Convention30_360),
            "Actual365" => Ok(InterestMethod::Actual365),
            "Actual360" => Ok(InterestMethod::Actual360),
            "ActualActual" => Ok(InterestMethod::ActualActual),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InterestType {
    Simple,
    Compound,
}
impl FromStr for InterestType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Simple" => Ok(InterestType::Simple),
            "Compound" => Ok(InterestType::Compound),
            _ => Err(()),
        }
    }
}

pub fn get_daily_interest_rate(annual_rate: Decimal, interest_method: InterestMethod) -> Decimal {
    let daily_rate: Decimal;

    match interest_method {
        InterestMethod::Convention30_360 => {
            daily_rate = annual_rate / Decimal::from(360);
        }
        InterestMethod::Actual365 => {
            daily_rate = annual_rate / Decimal::from(365);
        }
        InterestMethod::Actual360 => {
            daily_rate = annual_rate / Decimal::from(360);
        }
        InterestMethod::ActualActual => {
            daily_rate = annual_rate / Decimal::from(365); // adjusted later for leap years
        }
    }

    daily_rate
}

pub fn calculate_period_interest(
    start_date: NaiveDate,
    to_date: NaiveDate,
    payment_date: NaiveDate,
    daily_rate: Decimal,
    balance: Decimal,
    payment_amount: Decimal,
    interest_method: InterestMethod,
) -> (Decimal, u32) {
    let mut interest: Decimal;
    let mut days: u32;

    if interest_method == InterestMethod::Convention30_360 {
        interest = Decimal::from(30) * balance * daily_rate;
        days = 30;
    } else {
        days = 0;
        interest = Decimal::from(0);
        let mut current_date = start_date;

        let mut balance_m = balance;
        let mut daily_rate_m = daily_rate;
        while current_date <= to_date {
            if interest_method == InterestMethod::ActualActual && current_date.leap_year() {
                // Adjust daily rate for leap year
                daily_rate_m *= Decimal::from(365) / Decimal::from(366);
            }

            // Reduce balance on payment date
            if current_date == payment_date {
                balance_m -= payment_amount;
            }

            interest += balance_m * daily_rate_m;

            current_date = current_date + Days::new(1);
            days += 1;
        }
    }

    (round_decimal(
        interest,
        Some(INTEREST_PRECISION),
        Some(INTEREST_SCALE),
        Some(INTEREST_ROUNDING),
    ), days)
}

pub fn decompound_rate(annual_rate: Decimal) -> Decimal {
    let compounds_per_year = Decimal::from(12);
    let one = Decimal::ONE;

    let rate = ((one + annual_rate).powd(one / compounds_per_year) - one) * compounds_per_year;

    round_decimal(
        rate,
        Some(INTEREST_PRECISION),
        Some(6),
        Some(INTEREST_ROUNDING),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_get_daily_interest_rate() {
        let annual_rate = dec!(0.05); // 5% annual interest rate

        assert_eq!(
            get_daily_interest_rate(annual_rate, InterestMethod::Convention30_360),
            dec!(0.0001388888888888888888888889)
        );
        assert_eq!(
            get_daily_interest_rate(annual_rate, InterestMethod::Actual365),
            dec!(0.0001369863013698630136986301)
        );
        assert_eq!(
            get_daily_interest_rate(annual_rate, InterestMethod::Actual360),
            dec!(0.0001388888888888888888888889)
        );
        assert_eq!(
            get_daily_interest_rate(annual_rate, InterestMethod::ActualActual),
            dec!(0.0001369863013698630136986301)
        );
    }

    #[test]
    fn test_decompound_rate() {
        let rate = dec!(0.0512); // 5.12% EAR
        let decompounded_rate = decompound_rate(rate);

        assert_eq!(decompounded_rate, dec!(0.050036));
    }

    #[test]
    fn test_calculate_period_interest_convention30_360() {
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date");
        let to_date = NaiveDate::from_ymd_opt(2023, 1, 30).expect("Invalid date");
        let payment_date = NaiveDate::from_ymd_opt(2023, 1, 15).expect("Invalid date");
        let daily_rate = dec!(0.0001388888888888888888888889);
        let balance = dec!(1000);
        let payment_amount = dec!(100);
        let interest_method = InterestMethod::Convention30_360;

        let (interest, days) = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(4.17));
        assert_eq!(days, 30);
    }

    #[test]
    fn test_calculate_period_interest_actual365() {
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date");
        let to_date = NaiveDate::from_ymd_opt(2023, 1, 30).expect("Invalid date");
        let payment_date = NaiveDate::from_ymd_opt(2023, 1, 15).expect("Invalid date");
        let daily_rate = dec!(0.0001369863013698630136986301);
        let balance = dec!(1000);
        let payment_amount = dec!(100);
        let interest_method = InterestMethod::Actual365;

        let (interest, days) = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(3.89));
        assert_eq!(days, 30);
    }

    #[test]
    fn test_calculate_period_interest_actual360() {
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date");
        let to_date = NaiveDate::from_ymd_opt(2023, 1, 30).expect("Invalid date");
        let payment_date = NaiveDate::from_ymd_opt(2023, 1, 15).expect("Invalid date");
        let daily_rate = dec!(0.0001388888888888888888888889);
        let balance = dec!(1000);
        let payment_amount = dec!(100);
        let interest_method = InterestMethod::Actual360;

        let (interest, days) = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(3.94));
        assert_eq!(days, 30);

    }

    #[test]
    fn test_calculate_period_interest_actualactual() {
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date");
        let to_date = NaiveDate::from_ymd_opt(2023, 1, 30).expect("Invalid date");
        let payment_date = NaiveDate::from_ymd_opt(2023, 1, 15).expect("Invalid date");
        let daily_rate = dec!(0.0001369863013698630136986301);
        let balance = dec!(1000);
        let payment_amount = dec!(100);
        let interest_method = InterestMethod::ActualActual;

        let (interest, days) = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(3.89));
        assert_eq!(days, 30);

    }
}
