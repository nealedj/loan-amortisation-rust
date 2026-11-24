use super::interest::{
    calculate_period_interest, decompound_rate, get_daily_interest_rate, InterestMethod,
    InterestType,
};
use super::utils::round_decimal;
use chrono::{Days, Months, NaiveDate};

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub enum PaymentType {
    /// One of the scheduled periodic payments.
    Regular,
    /// A lump sum paid on the same date as, and in addition to, the final regular payment.
    Balloon,
}

#[derive(Debug, Serialize, Clone)]
pub struct Payment {
    pub month: u32,
    pub date: NaiveDate,
    pub payment_type: PaymentType,
    /// Total amount paid: `principal + interest + fee`.
    pub payment: Decimal,
    pub principal: Decimal,
    pub interest: Decimal,
    pub fee: Decimal,
    pub balance: Decimal,
    pub days: u32,
}
#[derive(Debug, Serialize)]

pub struct Meta {
    pub total_payable: Decimal,
    pub total_principal: Decimal,
    pub total_interest: Decimal,
    pub total_fees: Decimal,
    pub daily_rate: Decimal,
    pub annual_rate: Decimal,
    /// Annual percentage rate: the annual cost of credit, including fees.
    pub calculated_apr: Decimal,
    /// Effective annual rate of interest, excluding fees.
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
                total_fees: Decimal::from(0),
                daily_rate: Decimal::from(0),
                annual_rate: Decimal::from(0),
                calculated_apr: Decimal::from(0),
                calculated_ear: Decimal::from(0),
            },
        }
    }

    fn push(&mut self, payment: Payment) {
        self.meta.total_payable += payment.payment;
        self.meta.total_principal += payment.principal;
        self.meta.total_interest += payment.interest;
        self.meta.total_fees += payment.fee;
        self.payments.push(payment);
    }
}

/// Builds a schedule of `num_payments` regular payments of `period_payment`.
///
/// * `settle_balance` - adjust the final regular payment so that the loan is fully repaid
///   (leaving exactly the balloon outstanding for the balloon payment, if there is one).
/// * `balloon_payment` - an additional payment, made on the same date as the final regular
///   payment, which appears as a separate entry at the end of the schedule.
/// * `option_fee` - a fee added to the last payment in the schedule (the balloon payment,
///   if there is one).
///
/// The APR and EAR are not calculated here; see [`super::apr::calculate_annual_rate`].
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

    let balloon_amount = balloon_payment.unwrap_or(Decimal::ZERO);
    let option_fee = option_fee.unwrap_or(Decimal::ZERO);

    let mut balance = principal;
    let mut interest_payable_from = disbursal_date;
    let mut last_payment_date = first_payment_date;

    for month in 1..=num_payments {
        let is_final = month == num_payments;

        // Derive each date from the first one, rather than from the previous date, so that
        // month-end dates don't drift (31 Jan -> 28 Feb -> 31 Mar, not 28 Mar).
        let cap_date = first_capitalisation_date + Months::new(month - 1);
        let payment_date = first_payment_date + Months::new(month - 1);

        // The balloon is paid on the same date as the final regular payment, so from that
        // date interest accrues on the balance after both payments.
        let amount_paid_on_payment_date = if is_final {
            period_payment + balloon_amount
        } else {
            period_payment
        };

        let (interest, days) = calculate_period_interest(
            interest_payable_from,
            cap_date,
            payment_date,
            daily_rate,
            balance,
            amount_paid_on_payment_date,
            interest_method,
        );

        let principal_payment = if settle_balance && is_final {
            balance - balloon_amount
        } else {
            round_decimal(period_payment - interest, None, None, None)
        };
        let fee = if is_final && balloon_payment.is_none() {
            option_fee
        } else {
            Decimal::ZERO
        };

        balance = round_decimal(balance - principal_payment, None, None, None);

        schedule.push(Payment {
            month,
            date: payment_date,
            payment_type: PaymentType::Regular,
            payment: principal_payment + interest + fee,
            principal: principal_payment,
            interest,
            fee,
            balance,
            days,
        });

        interest_payable_from = cap_date + Days::new(1);
        last_payment_date = payment_date;
    }

    if let Some(balloon) = balloon_payment {
        balance = round_decimal(balance - balloon, None, None, None);

        schedule.push(Payment {
            month: num_payments,
            date: last_payment_date,
            payment_type: PaymentType::Balloon,
            payment: balloon + option_fee,
            principal: balloon,
            interest: Decimal::ZERO,
            fee: option_fee,
            balance,
            days: 0,
        });
    }

    schedule
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

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

        // Spans the 2024 leap year, where Actual/Actual accrues 1/366 of the rate per day
        assert_eq!(schedule.payments.len(), 36);
        assert_eq!(schedule.payments.last().unwrap().balance, Decimal::from(0));
        assert_eq!(
            schedule.meta.total_payable,
            Decimal::from_str("17105.59").unwrap()
        );
        assert_eq!(
            schedule.meta.total_principal,
            Decimal::from_str("15000").unwrap()
        );
        assert_eq!(
            schedule.meta.total_interest,
            Decimal::from_str("2105.59").unwrap()
        );
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn assert_components_add_up(schedule: &Schedule) {
        for p in &schedule.payments {
            assert_eq!(p.payment, p.principal + p.interest + p.fee, "row {:?}", p);
        }
        let meta = &schedule.meta;
        assert_eq!(
            meta.total_payable,
            meta.total_principal + meta.total_interest + meta.total_fees
        );
    }

    #[test]
    fn test_balloon_is_an_additional_payment() {
        let schedule = build_schedule(
            dec!(20000),
            date(2025, 1, 1),
            date(2025, 2, 1),
            date(2025, 2, 1),
            12,
            dec!(0.05),
            dec!(700),
            InterestMethod::Actual365,
            InterestType::Simple,
            true,
            Some(dec!(12000)),
            Some(dec!(10)),
        );

        assert_eq!(schedule.payments.len(), 13);
        assert_components_add_up(&schedule);

        let final_regular = &schedule.payments[11];
        assert_eq!(final_regular.payment_type, PaymentType::Regular);
        assert_eq!(final_regular.fee, Decimal::ZERO);
        // Settled so that exactly the balloon is left to pay
        assert_eq!(final_regular.balance, dec!(12000));

        let balloon = &schedule.payments[12];
        assert_eq!(balloon.payment_type, PaymentType::Balloon);
        assert_eq!(balloon.month, 12);
        assert_eq!(balloon.date, final_regular.date);
        assert_eq!(balloon.principal, dec!(12000));
        assert_eq!(balloon.interest, Decimal::ZERO);
        assert_eq!(balloon.fee, dec!(10));
        assert_eq!(balloon.payment, dec!(12010));
        assert_eq!(balloon.balance, Decimal::ZERO);
    }

    #[test]
    fn test_option_fee_added_to_final_fixed_payment() {
        let schedule = build_schedule(
            dec!(15000),
            date(2025, 1, 1),
            date(2025, 2, 1),
            date(2025, 2, 1),
            6,
            dec!(0.05),
            dec!(2537),
            InterestMethod::ActualActual,
            InterestType::Simple,
            false,
            None,
            Some(dec!(199)),
        );

        assert_components_add_up(&schedule);
        let last = schedule.payments.last().unwrap();
        assert_eq!(last.fee, dec!(199));
        assert_eq!(last.payment, dec!(2537) + dec!(199));
        assert_eq!(schedule.meta.total_fees, dec!(199));
        for p in &schedule.payments[..5] {
            assert_eq!(p.payment, dec!(2537));
            assert_eq!(p.fee, Decimal::ZERO);
        }
    }

    #[test]
    fn test_month_end_dates_do_not_drift() {
        let schedule = build_schedule(
            dec!(1000),
            date(2024, 12, 31),
            date(2025, 1, 31),
            date(2025, 1, 31),
            5,
            dec!(0.05),
            dec!(200),
            InterestMethod::Actual365,
            InterestType::Simple,
            true,
            None,
            None,
        );

        let dates: Vec<NaiveDate> = schedule.payments.iter().map(|p| p.date).collect();
        assert_eq!(
            dates,
            vec![
                date(2025, 1, 31),
                date(2025, 2, 28),
                date(2025, 3, 31),
                date(2025, 4, 30),
                date(2025, 5, 31)
            ]
        );
        // Interest periods follow the capitalisation dates: Feb 1-28, Mar 1-31, Apr 1-30, ...
        let days: Vec<u32> = schedule.payments.iter().map(|p| p.days).collect();
        assert_eq!(days, vec![32, 28, 31, 30, 31]);
    }
}
