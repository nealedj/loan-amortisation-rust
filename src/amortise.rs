mod interest;
mod schedule;
mod secant;
mod utils;

use chrono::NaiveDate;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

pub use interest::InterestMethod;
pub use interest::InterestType;
use schedule::build_schedule;
pub use schedule::Payment;
pub use schedule::Schedule;
use secant::secant_method;
use utils::round_decimal;

const PERIODS_PER_YEAR: u32 = 12;
const ESTIMATE_WINDOW: f32 = 2.5;

pub fn amortise(
    principal: Decimal,
    annual_rate: Decimal,
    num_payments: u32,
    disbursal_date: NaiveDate,
    first_payment_date: NaiveDate,
    first_capitalisation_date: NaiveDate,
    interest_method: InterestMethod,
    interest_type: InterestType,
    fixed_payment: Option<Decimal>,
) -> Schedule {
    let period_payment = if let Some(fixed_payment) = fixed_payment {
        // Use the provided fixed payment amount
        fixed_payment
    } else {
        // Calculate payment using secant method as before
        let mut period_payment = calculate_rough_period_payment(principal, annual_rate, num_payments);

        let f = |period_payment| {
            println!("Trying period payment: {}", period_payment);
            let schedule = build_schedule(
                principal,
                disbursal_date,
                first_capitalisation_date,
                first_payment_date,
                num_payments,
                annual_rate,
                period_payment,
                interest_method,
                interest_type,
                false,
            );
            schedule.payments.last().unwrap().balance // final balance
        };

        let estimate_window = Decimal::from_f32(ESTIMATE_WINDOW).unwrap();
        period_payment = match secant_method(
            f,
            period_payment / estimate_window,
            period_payment * estimate_window,
            Decimal::new(1, 2),
            4,
        ) {
            Some(root) => root,
            None => {
                println!("Failed to converge");
                return Schedule::new();
            }
        };

        round_decimal(period_payment, None, None, None)
    };

    let schedule = build_schedule(
        principal,
        disbursal_date,
        first_capitalisation_date,
        first_payment_date,
        num_payments,
        annual_rate,
        period_payment,
        interest_method,
        interest_type,
        fixed_payment.is_none(), // Only settle balance if we calculated the payment
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
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_rough_period_payment() {
        let principal = Decimal::from(15000);
        let annual_rate = Decimal::from_f64(8.9).unwrap() / Decimal::from(100);
        let num_payments = 36;

        let period_payment = calculate_rough_period_payment(principal, annual_rate, num_payments);

        assert!(period_payment > Decimal::from(0));
    }

    #[test]
    fn test_amortise_with_fixed_payment() {
        let principal = dec!(15000);
        let annual_rate = dec!(0.05); // 5% annual rate
        let num_payments = 6;
        let fixed_payment = dec!(2700);
        
        let disbursal_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let first_payment_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let first_capitalisation_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();

        // Test with fixed payment
        let schedule_fixed = amortise(
            principal,
            annual_rate,
            num_payments,
            disbursal_date,
            first_payment_date,
            first_capitalisation_date,
            InterestMethod::ActualActual,
            InterestType::Simple,
            Some(fixed_payment),
        );

        // Test without fixed payment (calculated payment)
        let schedule_calculated = amortise(
            principal,
            annual_rate,
            num_payments,
            disbursal_date,
            first_payment_date,
            first_capitalisation_date,
            InterestMethod::ActualActual,
            InterestType::Simple,
            None,
        );

        // Verify fixed payment schedule properties
        assert_eq!(schedule_fixed.payments.len(), num_payments as usize);
        
        // All payments except potentially the last should equal the fixed amount
        for (i, payment) in schedule_fixed.payments.iter().enumerate() {
            if i < (num_payments - 1) as usize {
                assert_eq!(payment.payment, fixed_payment);
            }
            assert!(payment.principal > Decimal::ZERO);
            assert!(payment.interest >= Decimal::ZERO);
        }

        // With a high fixed payment ($2700), the loan should be paid off early
        // (final balance should be negative, indicating overpayment)
        let final_balance = schedule_fixed.payments.last().unwrap().balance;
        assert!(final_balance < Decimal::ZERO, "Expected negative final balance with high fixed payment");

        // Verify calculated payment schedule balances to zero
        let calculated_final_balance = schedule_calculated.payments.last().unwrap().balance;
        assert_eq!(calculated_final_balance, Decimal::ZERO, "Calculated payment should result in zero final balance");

        // Fixed payment should be different from calculated payment
        let calculated_payment = schedule_calculated.payments[0].payment;
        assert_ne!(fixed_payment, calculated_payment, "Fixed payment should differ from calculated payment");
    }

    #[test]
    fn test_amortise_with_low_fixed_payment() {
        let principal = dec!(15000);
        let annual_rate = dec!(0.05);
        let num_payments = 6;
        let low_fixed_payment = dec!(2000); // Lower than optimal payment
        
        let disbursal_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let first_payment_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let first_capitalisation_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();

        let schedule = amortise(
            principal,
            annual_rate,
            num_payments,
            disbursal_date,
            first_payment_date,
            first_capitalisation_date,
            InterestMethod::ActualActual,
            InterestType::Simple,
            Some(low_fixed_payment),
        );

        // With low fixed payment, loan should not be fully paid off
        let final_balance = schedule.payments.last().unwrap().balance;
        assert!(final_balance > Decimal::ZERO, "Expected positive remaining balance with low fixed payment");

        // All payments should equal the fixed amount
        for payment in &schedule.payments {
            assert_eq!(payment.payment, low_fixed_payment);
        }
    }
}
