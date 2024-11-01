use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::str::FromStr;
use chrono::{NaiveDate, Months, Days};

const DEFAULT_SCALE: u32 = 2;
const DEFAULT_PRECISION: u32 = 28;
const DEFAULT_ROUNDING: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;
const PERIODS_PER_YEAR: u32 = 12;

fn main() {
    println!("Loan Amortisation Schedule Calculator");

    // Get loan details from user
    // let principal = get_input("Enter loan amount: ");
    // let annual_rate = get_input("Enter annual interest rate (as a percentage): ") / 100.0;
    // let years = get_input("Enter loan term in years: ");
    let principal = Decimal::from(15000);
    let annual_rate = Decimal::from_str("8.9").unwrap() / Decimal::from(100);
    let num_payments = 36;
    let disbursal_date = NaiveDate::from_ymd_opt(2024, 11, 1).unwrap();
    let first_payment_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    let first_capitalisation_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();

    // Convert basis points to period rate, scaled
    let period_rate = annual_rate / Decimal::from(PERIODS_PER_YEAR);
    let daily_rate = annual_rate / Decimal::from(365);

    let mut period_payment = calculate_rough_period_payment(principal, period_rate, num_payments);
    println!("\nperiod Payment: ${:.2}", period_payment);
    println!("\nAmortisation Schedule:");
    println!("Period | Payment | Principal | Interest | Remaining Balance");

    // let balance = build_schedule(principal, disbursal_date, first_capitalisation_date, first_payment_date, num_payments, daily_rate, period_payment);

    let f = |period_payment| build_schedule(principal, disbursal_date, first_capitalisation_date, first_payment_date, num_payments, daily_rate, period_payment);

    match secant_method(f, period_payment, period_payment * Decimal::new(1, 2), Decimal::new(1, 6), 100) {
        Some(root) => period_payment = root,
        None => println!("Failed to converge"),
    }

    build_schedule(principal, disbursal_date, first_capitalisation_date, first_payment_date, num_payments, daily_rate, period_payment);

}

fn build_schedule(principal: Decimal, disbursal_date: NaiveDate, first_capitalisation_date: NaiveDate, first_payment_date: NaiveDate, num_payments: i32, daily_rate: Decimal, period_payment: Decimal) -> Decimal {
    let mut balance = principal;
    let mut interest_payable_from = disbursal_date;
    let mut next_cap_date = first_capitalisation_date;
    let mut next_payment_date = first_payment_date;

    for month in 1..=num_payments {
        let interest = calculate_period_interest(interest_payable_from, next_cap_date, next_payment_date, daily_rate, balance, period_payment);
        let principal_payment = round_decimal(period_payment - interest, None, None, None);
        balance = round_decimal(balance-principal_payment, None, None, None);
    
        print_row(month, period_payment, principal_payment, interest, balance);
    
        interest_payable_from = next_cap_date + Days::new(1);
        next_cap_date = next_cap_date + Months::new(1);
        next_payment_date = next_payment_date + Months::new(1);
    
        if balance < Decimal::from_str("0.01").unwrap() {
            break;
        }
    }

    balance
}


fn calculate_period_interest(start_date: NaiveDate, to_date: NaiveDate, payment_date: NaiveDate, daily_rate: Decimal, balance: Decimal, payment_amount: Decimal) -> Decimal {
    let mut current_date = start_date;
    let mut interest = Decimal::from(0);

    let mut balance_m = balance;
    let mut daily_rate_m = daily_rate;
    while current_date <= to_date {

        if current_date.leap_year() {
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

    interest
}

fn calculate_rough_period_payment(principal: Decimal, period_rate: Decimal, num_payments: i32) -> Decimal {
    let one = Decimal::from(1);
    let factor = (one + period_rate).powd(Decimal::from(num_payments));
    round_decimal((principal * period_rate * factor) / (factor - one), None, None, None)
}

fn round_decimal(value: Decimal, precision: Option<u32>, scale: Option<u32>, rounding: Option<RoundingStrategy>) -> Decimal {
    let precision = precision.unwrap_or(DEFAULT_PRECISION);
    let scale = scale.unwrap_or(DEFAULT_SCALE);
    let rounding = rounding.unwrap_or(DEFAULT_ROUNDING);
    value.round_dp_with_strategy(scale.min(precision), rounding)
}

fn print_row(month: i32, payment: Decimal, principal: Decimal, interest: Decimal, balance: Decimal) {
    println!("{:5} | {:7.2} | {:9.2} | {:8.2} | {:17.2}", 
             month, payment, principal, interest, balance);
}

fn secant_method<F>(f: F, x0: Decimal, x1: Decimal, epsilon: Decimal, max_iterations: usize) -> Option<Decimal>
where
    F: Fn(Decimal) -> Decimal,
{
    let mut x0 = x0;
    let mut x1 = x1;
    let mut iteration = 0;

    while iteration < max_iterations {
        let f0 = f(x0);
        let f1 = f(x1);

        // Check if we've found a root
        if f1.abs() < epsilon {
            return Some(x1);
        }

        // Calculate the next x value
        let x2 = x1 - f1 * (x1 - x0) / (f1 - f0);

        // Check for convergence
        if (x2 - x1).abs() < epsilon {
            return Some(x2);
        }

        // Update values for next iteration
        x0 = x1;
        x1 = x2;
        iteration += 1;
    }

    // If we've reached here, the method didn't converge
    None
}