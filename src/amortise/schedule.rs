use super::interest::{
    calculate_period_interest, decompound_rate, get_daily_interest_rate, InterestMethod,
    InterestType,
};
use super::utils::round_decimal;
use chrono::{Days, Months, NaiveDate};

use rust_decimal::{Decimal, MathematicalOps};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Payment {
    pub month: u32,
    pub payment: Decimal,
    pub principal: Decimal,
    pub interest: Decimal,
    pub balance: Decimal,
    pub days: u32,
}
#[derive(Debug, Serialize)]

pub struct Meta {
    pub total_payable: Decimal,
    pub total_principal: Decimal,
    pub total_interest: Decimal,
    pub daily_rate: Decimal,
    pub annual_rate: Decimal,
    pub calculated_apr: Decimal,
    pub calculated_ear: Decimal,
}
#[derive(Debug, Serialize)]
pub struct Schedule {
    pub payments: Vec<Payment>,
    pub meta: Meta,
}

impl Schedule {
    pub fn new() -> Self {
        Schedule {
            payments: Vec::new(),
            meta: Meta {
                total_payable: Decimal::from(0),
                total_principal: Decimal::from(0),
                total_interest: Decimal::from(0),
                daily_rate: Decimal::from(0),
                annual_rate: Decimal::from(0),
                calculated_apr: Decimal::from(0),
                calculated_ear: Decimal::from(0),
            },
        }
    }
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
    interest_type: InterestType,
    settle_balance: bool,
    balloon_payment: Option<Decimal>,
    option_fee: Option<Decimal>,
) -> Schedule {
    let mut schedule = Schedule::new();

    if interest_type == InterestType::Compound {
        schedule.meta.annual_rate = decompound_rate(annual_rate);
    } else {
        schedule.meta.annual_rate = annual_rate;
    }

    let daily_rate = get_daily_interest_rate(schedule.meta.annual_rate, interest_method);
    schedule.meta.daily_rate = daily_rate;

    let mut balance = principal;
    let mut interest_payable_from = disbursal_date;
    let mut next_cap_date = first_capitalisation_date;
    let mut next_payment_date = first_payment_date;

    for month in 1..=num_payments {
        let (interest, days) = calculate_period_interest(
            interest_payable_from,
            next_cap_date,
            next_payment_date,
            daily_rate,
            balance,
            period_payment,
            interest_method,
        );
        let mut principal_payment;
        let mut payment;

        if settle_balance && month == num_payments {
            // For the final payment, add any remaining balance
            payment = balance + interest;
            // Add option fee to final payment if present
            if let Some(fee) = option_fee {
                payment += fee;
            }
        } else if month == num_payments && balloon_payment.is_some() {
            // Final payment with balloon payment - the payment IS the balloon payment amount
            payment = balloon_payment.unwrap();
            // Add option fee to balloon payment if present
            if let Some(fee) = option_fee {
                payment += fee;
            }
        } else {
            payment = period_payment;
        }

        principal_payment = round_decimal(payment - interest, None, None, None);
        
        // Adjust principal payment to exclude option fee
        if month == num_payments && option_fee.is_some() {
            principal_payment = round_decimal(principal_payment - option_fee.unwrap(), None, None, None);
        }
        
        // For balloon payments, adjust the principal payment calculation
        if month == num_payments && balloon_payment.is_some() && !settle_balance {
            // Final balloon payment: the principal payment should clear the remaining balance
            principal_payment = balance;
            // The interest for the final payment should be calculated normally
            // The "payment" amount is the balloon payment amount, but for accounting purposes
            // we track principal and interest separately
        }

        balance = round_decimal(balance - principal_payment, None, None, None);

        schedule.payments.push(Payment {
            month,
            payment: payment,
            principal: principal_payment,
            interest,
            balance,
            days,
        });
        schedule.meta.total_payable += payment;
        schedule.meta.total_principal += principal_payment;
        schedule.meta.total_interest += interest;

        interest_payable_from = next_cap_date + Days::new(1);
        next_cap_date = next_cap_date + Months::new(1);
        next_payment_date = next_payment_date + Months::new(1);
    }

    schedule.meta.calculated_apr = get_apr(schedule.payments.clone());
    schedule.meta.calculated_ear = schedule.meta.calculated_apr; // TODO: revise once fees are added

    schedule
}

fn get_apr(payments: Vec<Payment>) -> Decimal {
    let mut balance_curve = Decimal::from(0);
    let mut total_interest = Decimal::from(0);

    for payment in payments {
        balance_curve += payment.balance * Decimal::from(payment.days);
        total_interest += payment.interest;
    }

    let daily_cost = total_interest / balance_curve;
    let payments_per_year = Decimal::from(12);

    let apr = (Decimal::ONE + daily_cost * Decimal::from(365) / payments_per_year)
        .powd(payments_per_year)
        - Decimal::ONE;

    round_decimal(apr, None, Some(6), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    #[test]
    fn test_get_apr() {
        let principal = Decimal::from(15000);
        let annual_rate = dec!(0.05);

        let num_payments = 24;
        let period_payment = dec!(476.3);

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
            InterestType::Simple,
            true,
            None, // no balloon payment
            None, // no option fee
        );

        let apr = get_apr(schedule.payments);

        assert_eq!(apr, dec!(0.053803));
    }

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
            InterestType::Simple,
            true,
            None, // no balloon payment
            None, // no option fee
        );

        assert_eq!(schedule.payments.len(), 36);
        assert_eq!(schedule.payments.last().unwrap().balance, Decimal::from(0));
        assert_eq!(
            schedule.meta.total_payable,
            Decimal::from_str("17073.12").unwrap()
        );
        assert_eq!(
            schedule.meta.total_principal,
            Decimal::from_str("15000").unwrap()
        );
        assert_eq!(
            schedule.meta.total_interest,
            Decimal::from_str("2073.12").unwrap()
        );
    }
}
