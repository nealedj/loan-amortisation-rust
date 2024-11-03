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
