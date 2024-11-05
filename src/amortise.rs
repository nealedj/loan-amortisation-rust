mod interest;
mod schedule;
mod secant;
mod utils;

use chrono::NaiveDate;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

pub use interest::InterestMethod;
pub use schedule::Payment;
use schedule::build_schedule;
use secant::secant_method;
use utils::round_decimal;

const PERIODS_PER_YEAR: u32 = 12;

pub fn amortise(
    principal: Decimal,
    annual_rate: Decimal,
    num_payments: u32,
    disbursal_date: NaiveDate,
    first_payment_date: NaiveDate,
    first_capitalisation_date: NaiveDate,
    interest_method: InterestMethod,
) -> Vec<Payment> {
    let mut period_payment = calculate_rough_period_payment(principal, annual_rate, num_payments);

    let f = |period_payment| {
        let schedule = build_schedule(
            principal,
            disbursal_date,
            first_capitalisation_date,
            first_payment_date,
            num_payments,
            annual_rate,
            period_payment,
            interest_method,
            false,
        );
        schedule.last().unwrap().balance // final balance
    };

    match secant_method(
        f,
        period_payment,
        period_payment * Decimal::new(1, 2),
        Decimal::new(1, 6),
        100,
    ) {
        Some(root) => period_payment = root,
        None => println!("Failed to converge"),
    }

    period_payment = round_decimal(period_payment, None, None, None);
    let schedule = build_schedule(
        principal,
        disbursal_date,
        first_capitalisation_date,
        first_payment_date,
        num_payments,
        annual_rate,
        period_payment,
        interest_method,
        true,
    );

    schedule
}

fn calculate_rough_period_payment(
    principal: Decimal,
    annual_rate: Decimal,
    num_payments: u32,
) -> Decimal {
    let one = Decimal::from(1);
    let period_rate = annual_rate / Decimal::from(PERIODS_PER_YEAR);
    let factor = (one + period_rate).powd(Decimal::from(num_payments));
    round_decimal(
        (principal * period_rate * factor) / (factor - one),
        None,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rough_period_payment() {
        let principal = Decimal::from(15000);
        let annual_rate = Decimal::from_f64(8.9).unwrap() / Decimal::from(100);
        let num_payments = 36;

        let period_payment = calculate_rough_period_payment(principal, annual_rate, num_payments);

        assert!(period_payment > Decimal::from(0));
    }
}