//! Annual rate calculations (APR / EAR) from a schedule's cash flows.
//!
//! The annual rate `X` is the rate at which the amount lent equals the present value of
//! everything the borrower repays (UK CONC App 1.1 / EU Consumer Credit Directive Annex I):
//!
//! ```text
//! principal = Σ amount_k · (1 + X)^(−t_k)
//! ```
//!
//! where `t_k` is the time in years from the disbursal date to repayment `k`. A year is
//! taken to be 365 days, or 366 days in a leap year.

use chrono::{Datelike, NaiveDate};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

use super::schedule::Payment;
use super::utils::round_decimal;

const RATE_SCALE: u32 = 6;
const MAX_ITERATIONS: usize = 100;
const TOLERANCE: Decimal = dec!(0.000000000001);

/// Calculates the annual rate implied by the payments in a schedule.
///
/// * `include_fees` - include fees in the repayments. `true` gives the APR (total cost of
///   credit); `false` gives the effective annual rate of interest alone.
///
/// Any balance left at the end of the schedule (e.g. from a fixed payment that is too low or
/// too high) is treated as settled, or refunded, on the final payment date.
///
/// Returns `None` if the rate cannot be determined (no payments, or no solution).
pub fn calculate_annual_rate(
    principal: Decimal,
    disbursal_date: NaiveDate,
    payments: &[Payment],
    include_fees: bool,
) -> Option<Decimal> {
    let last = payments.last()?;

    let mut cash_flows: Vec<(Decimal, Decimal)> = payments
        .iter()
        .map(|p| {
            let amount = if include_fees { p.payment } else { p.payment - p.fee };
            (year_fraction(disbursal_date, p.date), amount)
        })
        .collect();

    if !last.balance.is_zero() {
        cash_flows.push((year_fraction(disbursal_date, last.date), last.balance));
    }

    let rate = solve_annual_rate(principal, &cash_flows)?;
    Some(round_decimal(rate, None, Some(RATE_SCALE), None))
}

/// Solves `principal = Σ amount · (1 + X)^(−years)` for `X` using Newton's method.
///
/// Starting from zero, the iteration approaches the root from below and converges
/// monotonically for repayments that exceed the principal, because the present value is a
/// convex, decreasing function of the rate.
fn solve_annual_rate(principal: Decimal, cash_flows: &[(Decimal, Decimal)]) -> Option<Decimal> {
    if principal <= Decimal::ZERO || cash_flows.is_empty() {
        return None;
    }

    let mut rate = Decimal::ZERO;
    for _ in 0..MAX_ITERATIONS {
        let growth = Decimal::ONE + rate;
        let log_growth = growth.checked_ln()?;

        let mut npv = -principal;
        let mut slope = Decimal::ZERO;
        for &(years, amount) in cash_flows {
            let discounted = amount.checked_mul(discount_factor(years, log_growth)?)?;
            npv += discounted;
            slope -= years * discounted;
        }
        // d/dX (1 + X)^(−t) = −t · (1 + X)^(−t) / (1 + X)
        slope = slope.checked_div(growth)?;
        if slope.is_zero() {
            return None;
        }

        let mut next = rate - npv.checked_div(slope)?;
        if next <= -Decimal::ONE {
            // Stay within the domain of the function: step halfway towards -100%.
            next = (rate - Decimal::ONE) / Decimal::TWO;
        }

        if (next - rate).abs() < TOLERANCE {
            return Some(next);
        }
        rate = next;
    }

    None
}

/// `(1 + X)^(−years)`, given `ln(1 + X)`.
fn discount_factor(years: Decimal, log_growth: Decimal) -> Option<Decimal> {
    let exponent = -(years * log_growth);
    match exponent.checked_exp() {
        Some(factor) => Some(factor),
        // Too small to represent: the cash flow has no meaningful present value.
        None if exponent.is_sign_negative() => Some(Decimal::ZERO),
        None => None,
    }
}

/// Time between two dates in years, counting days in a leap year as 1/366 of a year and
/// all other days as 1/365.
pub fn year_fraction(start: NaiveDate, end: NaiveDate) -> Decimal {
    if end < start {
        return -year_fraction(end, start);
    }

    let mut total = Decimal::ZERO;
    let mut cursor = start;
    while cursor < end {
        let next_year = NaiveDate::from_ymd_opt(cursor.year() + 1, 1, 1).unwrap();
        let segment_end = next_year.min(end);
        let days_in_year = if cursor.leap_year() { 366 } else { 365 };
        total += Decimal::from((segment_end - cursor).num_days()) / Decimal::from(days_in_year);
        cursor = segment_end;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amortise::schedule::PaymentType;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn payment(date: NaiveDate, amount: Decimal, fee: Decimal, balance: Decimal) -> Payment {
        Payment {
            month: 1,
            date,
            payment_type: PaymentType::Regular,
            payment: amount,
            principal: Decimal::ZERO,
            interest: Decimal::ZERO,
            fee,
            balance,
            days: 0,
        }
    }

    #[test]
    fn test_year_fraction() {
        assert_eq!(year_fraction(date(2023, 1, 1), date(2024, 1, 1)), Decimal::ONE);
        assert_eq!(year_fraction(date(2024, 1, 1), date(2025, 1, 1)), Decimal::ONE);
        assert_eq!(year_fraction(date(2023, 1, 1), date(2025, 1, 1)), Decimal::TWO);
        assert_eq!(
            year_fraction(date(2023, 12, 1), date(2024, 2, 1)),
            Decimal::from(31) / Decimal::from(365) + Decimal::from(31) / Decimal::from(366)
        );
        assert_eq!(year_fraction(date(2023, 5, 5), date(2023, 5, 5)), Decimal::ZERO);
        assert_eq!(
            year_fraction(date(2024, 1, 1), date(2023, 1, 1)),
            -Decimal::ONE
        );
    }

    #[test]
    fn test_single_repayment_after_one_year() {
        let payments = [payment(date(2024, 1, 1), dec!(1050), Decimal::ZERO, Decimal::ZERO)];
        let rate = calculate_annual_rate(dec!(1000), date(2023, 1, 1), &payments, true);
        assert_eq!(rate, Some(dec!(0.05)));
    }

    #[test]
    fn test_repayments_over_two_years() {
        // 1000 at 10%: pay 600 after one year (leaving 500), then 550 after two years.
        let payments = [
            payment(date(2024, 1, 1), dec!(600), Decimal::ZERO, dec!(500)),
            payment(date(2025, 1, 1), dec!(550), Decimal::ZERO, Decimal::ZERO),
        ];
        let rate = calculate_annual_rate(dec!(1000), date(2023, 1, 1), &payments, true);
        assert_eq!(rate, Some(dec!(0.1)));
    }

    #[test]
    fn test_fees_included_only_in_apr() {
        let payments = [payment(date(2024, 1, 1), dec!(1100), dec!(50), Decimal::ZERO)];
        let apr = calculate_annual_rate(dec!(1000), date(2023, 1, 1), &payments, true);
        let ear = calculate_annual_rate(dec!(1000), date(2023, 1, 1), &payments, false);
        assert_eq!(apr, Some(dec!(0.1)));
        assert_eq!(ear, Some(dec!(0.05)));
    }

    #[test]
    fn test_remaining_balance_treated_as_settled() {
        // Only 500 repaid, 550 still owed at the end: same as repaying 1050 in total.
        let payments = [payment(date(2024, 1, 1), dec!(500), Decimal::ZERO, dec!(550))];
        let rate = calculate_annual_rate(dec!(1000), date(2023, 1, 1), &payments, true);
        assert_eq!(rate, Some(dec!(0.05)));
    }

    #[test]
    fn test_no_payments() {
        assert_eq!(
            calculate_annual_rate(dec!(1000), date(2023, 1, 1), &[], true),
            None
        );
    }
}
