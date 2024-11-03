use super::utils::round_decimal;
use chrono::{Days, NaiveDate};
use rust_decimal::prelude::RoundingStrategy;
use rust_decimal::Decimal;

const INTEREST_SCALE: u32 = 2;
const INTEREST_PRECISION: u32 = 28;
const INTEREST_ROUNDING: RoundingStrategy = RoundingStrategy::MidpointNearestEven;

#[derive(PartialEq, Copy, Clone)]
pub enum InterestMethod {
    Convention30_360,
    Actual365,
    Actual360,
    ActualActual,
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
) -> Decimal {
    if interest_method == InterestMethod::Convention30_360 {
        return Decimal::from(30) * balance * daily_rate;
    }

    let mut current_date = start_date;
    let mut interest = Decimal::from(0);

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

        current_date = current_date + Days::new(1)
    }

    round_decimal(
        interest,
        Some(INTEREST_PRECISION),
        Some(INTEREST_SCALE),
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
    fn test_calculate_period_interest_convention30_360() {
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date");
        let to_date = NaiveDate::from_ymd_opt(2023, 1, 30).expect("Invalid date");
        let payment_date = NaiveDate::from_ymd_opt(2023, 1, 15).expect("Invalid date");
        let daily_rate = dec!(0.0001388888888888888888888889);
        let balance = dec!(1000);
        let payment_amount = dec!(100);
        let interest_method = InterestMethod::Convention30_360;

        let interest = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(4.1666666666666666666666670000));
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

        let interest = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(3.89));
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

        let interest = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(3.94));
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

        let interest = calculate_period_interest(
            start_date,
            to_date,
            payment_date,
            daily_rate,
            balance,
            payment_amount,
            interest_method,
        );

        assert_eq!(interest, dec!(3.89));
    }
}
