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
    balloon_payment: Option<Decimal>,
    option_fee: Option<Decimal>,
) -> Schedule {
    let period_payment = if let Some(fixed_payment) = fixed_payment {
        // Use the provided fixed payment amount
        fixed_payment
    } else {
        // Calculate payment using secant method as before
        // For balloon payment scenarios, reduce the principal by the balloon amount for calculation
        let effective_principal = if let Some(balloon) = balloon_payment {
            principal - balloon
        } else {
            principal
        };
        
        let mut period_payment = calculate_rough_period_payment(effective_principal, annual_rate, num_payments);

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
                false, // Don't settle balance in secant method iterations
                None, // Don't apply balloon payment logic during secant iterations
                None, // Don't apply option fee during secant iterations
            );
            let final_balance = schedule.payments.last().unwrap().balance;
            // For balloon payment scenarios, we want the final balance to equal the balloon payment amount
            if let Some(balloon) = balloon_payment {
                final_balance - balloon // Target: balance should equal balloon payment
            } else {
                final_balance // Normal case: target is zero balance
            }
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
        fixed_payment.is_none() && balloon_payment.is_none(), // Only settle balance if we calculated the payment AND no balloon payment
        balloon_payment,
        option_fee,
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
            None, // no balloon payment
            None, // no option fee
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
            None, // no balloon payment
            None, // no option fee
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
            None, // no balloon payment
            None, // no option fee
        );

        // With low fixed payment, loan should not be fully paid off
        let final_balance = schedule.payments.last().unwrap().balance;
        assert!(final_balance > Decimal::ZERO, "Expected positive remaining balance with low fixed payment");

        // All payments should equal the fixed amount
        for payment in &schedule.payments {
            assert_eq!(payment.payment, low_fixed_payment);
        }
    }

    #[test]
    fn test_amortise_with_balloon_payment() {
        let principal = dec!(20000);
        let annual_rate = dec!(0.06); // 6% annual rate
        let num_payments = 12; // 1 year
        let balloon_payment = dec!(15000); // Large balloon payment
        
        let disbursal_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let first_payment_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let first_capitalisation_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();

        // Test with balloon payment - should have lower monthly payments
        let schedule_balloon = amortise(
            principal,
            annual_rate,
            num_payments,
            disbursal_date,
            first_payment_date,
            first_capitalisation_date,
            InterestMethod::ActualActual,
            InterestType::Simple,
            None, // calculated payment
            Some(balloon_payment),
            None, // no option fee
        );

        // Test without balloon payment for comparison
        let schedule_normal = amortise(
            principal,
            annual_rate,
            num_payments,
            disbursal_date,
            first_payment_date,
            first_capitalisation_date,
            InterestMethod::ActualActual,
            InterestType::Simple,
            None, // calculated payment
            None, // no balloon payment
            None, // no option fee
        );

        // Verify balloon payment schedule properties
        assert_eq!(schedule_balloon.payments.len(), num_payments as usize);
        
        // Monthly payments should be lower with balloon payment
        let balloon_monthly_payment = schedule_balloon.payments[0].payment;
        let normal_monthly_payment = schedule_normal.payments[0].payment;
        assert!(balloon_monthly_payment < normal_monthly_payment, 
                "Balloon payment schedule should have lower monthly payments");

        // Final payment should equal the balloon payment amount
        let final_payment = schedule_balloon.payments.last().unwrap();
        assert_eq!(final_payment.payment, balloon_payment, 
                "Final payment should equal balloon payment amount");

        // Final balance should be zero
        assert_eq!(final_payment.balance, Decimal::ZERO, 
                "Final balance should be zero with balloon payment");

        println!("Balloon monthly payment: {}", balloon_monthly_payment);
        println!("Normal monthly payment: {}", normal_monthly_payment);
        println!("Final payment: {}", final_payment.payment);
        println!("Final balance: {}", final_payment.balance);
    }

    #[test]
    fn test_amortise_with_fixed_payment_and_balloon() {
        let principal = dec!(25000);
        let annual_rate = dec!(0.05); // 5% annual rate
        let num_payments = 24; // 2 years
        let fixed_payment = dec!(800); // Fixed monthly payment
        let balloon_payment = dec!(10000); // Balloon payment
        
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
            Some(fixed_payment),
            Some(balloon_payment),
            None, // no option fee
        );

        // Verify schedule properties
        assert_eq!(schedule.payments.len(), num_payments as usize);
        
        // All payments except the last should equal the fixed amount
        for (i, payment) in schedule.payments.iter().enumerate() {
            if i < (num_payments - 1) as usize {
                assert_eq!(payment.payment, fixed_payment, 
                          "Payment {} should equal fixed payment amount", i + 1);
            }
        }

        // Final payment should equal the balloon payment
        let final_payment = schedule.payments.last().unwrap();
        assert_eq!(final_payment.payment, balloon_payment, 
                "Final payment should equal balloon payment amount");

        println!("Fixed payment: {}", fixed_payment);
        println!("Final payment: {}", final_payment.payment);
        println!("Final balance: {}", final_payment.balance);
    }

    #[test]
    fn test_pcp_scenario() {
        // Typical PCP (Personal Contract Purchase) scenario
        let vehicle_price = dec!(30000);
        let deposit = dec!(5000);
        let principal = vehicle_price - deposit; // £25,000 financed
        let annual_rate = dec!(0.049); // 4.9% APR
        let num_payments = 36; // 3 years
        let balloon_payment = dec!(12000); // GMFV (Guaranteed Minimum Future Value)
        
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
            None, // calculate optimal payment
            Some(balloon_payment),
            None, // no option fee
        );

        // Verify PCP schedule properties
        assert_eq!(schedule.payments.len(), num_payments as usize);
        
        // Monthly payments should be consistent (except final payment)
        let monthly_payment = schedule.payments[0].payment;
        for (i, payment) in schedule.payments.iter().enumerate() {
            if i < (num_payments - 1) as usize {
                assert!((payment.payment - monthly_payment).abs() < dec!(0.01), 
                       "Monthly payments should be consistent in PCP");
            }
        }

        // Final payment should equal the balloon payment
        let final_payment = schedule.payments.last().unwrap();
        println!("PCP Monthly payment: £{}", monthly_payment);
        println!("PCP Final payment: £{}", final_payment.payment);
        println!("PCP Balloon payment: £{}", balloon_payment);
        
        assert_eq!(final_payment.payment, balloon_payment, 
                "Final PCP payment should equal balloon payment amount");

        // Final balance should be zero
        assert_eq!(final_payment.balance, Decimal::ZERO, 
                "PCP should result in zero final balance");

        println!("PCP Monthly payment: £{}", monthly_payment);
        println!("PCP Final payment: £{}", final_payment.payment);
        println!("PCP Final balance: £{}", final_payment.balance);
        
        // Verify the monthly payment is reasonable for PCP (should be lower than traditional loan)
        assert!(monthly_payment < dec!(500), 
                "PCP monthly payment should be reasonable for a £25k vehicle");
    }

    #[test]
    fn test_amortise_with_option_fee() {
        let principal = dec!(15000);
        let annual_rate = dec!(0.08); // 8% annual rate
        let num_payments = 24; // 2 years
        let option_fee = dec!(199); // £199 option fee
        
        let disbursal_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let first_payment_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let first_capitalisation_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();

        // Test with option fee - should add fee to final payment
        let schedule_with_fee = amortise(
            principal,
            annual_rate,
            num_payments,
            disbursal_date,
            first_payment_date,
            first_capitalisation_date,
            InterestMethod::ActualActual,
            InterestType::Simple,
            None, // calculated payment
            None, // no balloon payment
            Some(option_fee),
        );

        // Test without option fee for comparison
        let schedule_without_fee = amortise(
            principal,
            annual_rate,
            num_payments,
            disbursal_date,
            first_payment_date,
            first_capitalisation_date,
            InterestMethod::ActualActual,
            InterestType::Simple,
            None, // calculated payment
            None, // no balloon payment
            None, // no option fee
        );

        // Verify option fee schedule properties
        assert_eq!(schedule_with_fee.payments.len(), num_payments as usize);
        assert_eq!(schedule_without_fee.payments.len(), num_payments as usize);
        
        // Monthly payments should be the same except for the final payment
        for i in 0..(num_payments - 1) as usize {
            let payment_with_fee = &schedule_with_fee.payments[i];
            let payment_without_fee = &schedule_without_fee.payments[i];
            assert!((payment_with_fee.payment - payment_without_fee.payment).abs() < dec!(0.01),
                   "Monthly payments should be approximately the same");
        }

        // Final payment should include the option fee
        let final_payment_with_fee = schedule_with_fee.payments.last().unwrap();
        let final_payment_without_fee = schedule_without_fee.payments.last().unwrap();
        
        let fee_difference = final_payment_with_fee.payment - final_payment_without_fee.payment;
        assert!((fee_difference - option_fee).abs() < dec!(0.01),
               "Final payment should include option fee. Expected difference: {}, Actual: {}", 
               option_fee, fee_difference);

        // Both should have zero final balance
        assert_eq!(final_payment_with_fee.balance, Decimal::ZERO);
        assert_eq!(final_payment_without_fee.balance, Decimal::ZERO);

        println!("Monthly payment: £{}", schedule_with_fee.payments[0].payment);
        println!("Final payment with fee: £{}", final_payment_with_fee.payment);
        println!("Final payment without fee: £{}", final_payment_without_fee.payment);
        println!("Option fee: £{}", option_fee);
    }

    #[test]
    fn test_hp_scenario_with_balloon_and_option_fee() {
        // HP (Hire Purchase) scenario with both balloon payment and option fee
        let vehicle_price = dec!(20000);
        let deposit = dec!(2000);
        let principal = vehicle_price - deposit; // £18,000 financed
        let annual_rate = dec!(0.054); // 5.4% APR
        let num_payments = 48; // 4 years
        let balloon_payment = dec!(6000); // Final payment
        let option_fee = dec!(299); // HP option fee
        
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
            None, // calculate optimal payment
            Some(balloon_payment),
            Some(option_fee),
        );

        // Verify HP schedule properties
        assert_eq!(schedule.payments.len(), num_payments as usize);
        
        // Monthly payments should be consistent (except final payment)
        let monthly_payment = schedule.payments[0].payment;
        for (i, payment) in schedule.payments.iter().enumerate() {
            if i < (num_payments - 1) as usize {
                assert!((payment.payment - monthly_payment).abs() < dec!(0.01), 
                       "Monthly payments should be consistent in HP");
            }
        }

        // Final payment should equal balloon payment + option fee
        let final_payment = schedule.payments.last().unwrap();
        let expected_final = balloon_payment + option_fee;
        assert_eq!(final_payment.payment, expected_final, 
                "Final HP payment should equal balloon payment + option fee");

        // Final balance should be zero
        assert_eq!(final_payment.balance, Decimal::ZERO, 
                "HP should result in zero final balance");

        println!("HP Monthly payment: £{}", monthly_payment);
        println!("HP Final payment: £{}", final_payment.payment);
        println!("HP Balloon payment: £{}", balloon_payment);
        println!("HP Option fee: £{}", option_fee);
        
        // Verify the monthly payment is reasonable
        assert!(monthly_payment < dec!(350), 
                "HP monthly payment should be reasonable for an £18k vehicle");
    }
}
