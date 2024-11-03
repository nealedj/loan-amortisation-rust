use super::interest::{calculate_period_interest, get_daily_interest_rate, InterestMethod};
use super::utils::round_decimal;
use chrono::{Days, Months, NaiveDate};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct Payment {
    pub month: u32,
    pub payment: Decimal,
    pub principal: Decimal,
    pub interest: Decimal,
    pub balance: Decimal,
}

pub fn build_schedule(
    principal: Decimal,
    disbursal_date: NaiveDate,
    first_capitalisation_date: NaiveDate,
    first_payment_date: NaiveDate,
    num_payments: u32,
    annual_rate: Decimal,
    period_payment: Decimal,
    interest_method: InterestMethod,
    settle_balance: bool,
) -> Vec<Payment> {
    let daily_rate = get_daily_interest_rate(annual_rate, interest_method);

    let mut schedule = Vec::new();
    let mut balance = principal;
    let mut interest_payable_from = disbursal_date;
    let mut next_cap_date = first_capitalisation_date;
    let mut next_payment_date = first_payment_date;

    for month in 1..=num_payments {
        let interest = calculate_period_interest(
            interest_payable_from,
            next_cap_date,
            next_payment_date,
            daily_rate,
            balance,
            period_payment,
            interest_method,
        );
        let principal_payment;

        if settle_balance && month == num_payments {
            principal_payment = balance;
        } else {
            principal_payment = round_decimal(period_payment - interest, None, None, None);
        }

        balance = round_decimal(balance - principal_payment, None, None, None);

        schedule.push(Payment {
            month,
            payment: period_payment,
            principal: principal_payment,
            interest,
            balance,
        });

        interest_payable_from = next_cap_date + Days::new(1);
        next_cap_date = next_cap_date + Months::new(1);
        next_payment_date = next_payment_date + Months::new(1);

        if balance < Decimal::from_str("0.01").unwrap() {
            break;
        }
    }

    schedule
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_schedule() {
        let principal = Decimal::from(15000);
        let annual_rate = Decimal::from_str("8.9").unwrap() / Decimal::from(100);

        let num_payments = 36;
        let period_payment = Decimal::from_str("476.3").unwrap();

        let disbursal_date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        let first_capitalisation_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let first_payment_date = NaiveDate::from_ymd_opt(2023, 3, 1).unwrap();

        let schedule = build_schedule(
            principal,
            disbursal_date,
            first_capitalisation_date,
            first_payment_date,
            num_payments,
            annual_rate,
            period_payment,
            InterestMethod::ActualActual,
            true,
        );

        assert_eq!(schedule.len(), 36);
        assert_eq!(schedule.last().unwrap().balance, Decimal::from(0));
    }

    #[test]
    fn test_get_daily_interest_rate() {
        let annual_rate = Decimal::from_f64(8.9).unwrap() / Decimal::from(100);
        let interest_method = InterestMethod::ActualActual;

        let daily_rate = get_daily_interest_rate(annual_rate, interest_method);

        assert!(daily_rate > Decimal::from(0));
    }
}
