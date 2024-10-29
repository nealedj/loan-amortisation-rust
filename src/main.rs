use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::io;
use std::str::FromStr;

const DEFAULT_SCALE: u32 = 2;
const DEFAULT_PRECISION: u32 = 28;
const DEFAULT_ROUNDING: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;

fn main() {
    println!("Loan Amortisation Schedule Calculator");

    // Get loan details from user
    // let principal = get_input("Enter loan amount: ");
    // let annual_rate = get_input("Enter annual interest rate (as a percentage): ") / 100.0;
    // let years = get_input("Enter loan term in years: ");
    let principal = Decimal::from(15000); // convert to pence
    let annual_rate = Decimal::from_str("8.9").unwrap() / Decimal::from(100); // convert to basis points
    let num_payments = 36;

    // Calculate monthly payment (in pence)
    // Convert basis points to monthly rate, scaled
    let monthly_rate = annual_rate / Decimal::from(12);

    let monthly_payment = calculate_monthly_payment(principal, monthly_rate, num_payments);
    println!("\nMonthly Payment: ${:.2}", monthly_payment);
    println!("\nAmortization Schedule:");
    println!("Month | Payment | Principal | Interest | Remaining Balance");

    let mut balance = principal;
    for month in 1..=num_payments {
        let interest = round_decimal(balance * monthly_rate, None, None, None);
        let principal_payment = round_decimal(monthly_payment - interest, None, None, None);
        balance = round_decimal(balance-principal_payment, None, None, None);

        print_row(month, monthly_payment, principal_payment, interest, balance);

        if balance < Decimal::from_str("0.01").unwrap() {
            break;
        }
    }
}

fn calculate_monthly_payment(principal: Decimal, monthly_rate: Decimal, num_payments: i32) -> Decimal {
    let one = Decimal::from(1);
    let factor = (one + monthly_rate).powd(Decimal::from(num_payments));
    round_decimal((principal * monthly_rate * factor) / (factor - one), None, None, None)
}

fn round_decimal(value: Decimal, precision: Option<u32>, scale: Option<u32>, rounding: Option<RoundingStrategy>) -> Decimal {
    let precision = precision.unwrap_or(DEFAULT_PRECISION);
    let scale = scale.unwrap_or(DEFAULT_SCALE);
    let rounding = rounding.unwrap_or(DEFAULT_ROUNDING);
    value.round_dp_with_strategy(scale.min(precision), rounding)
}

fn print_row(month: i32, payment: Decimal, principal: Decimal, interest: Decimal, balance: Decimal) {
    println!("{:5} | ${:7.2} | ${:9.2} | ${:8.2} | ${:17.2}", 
             month, payment, principal, interest, balance);
}

fn get_decimal_input(prompt: &str) -> Decimal {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        match Decimal::from_str(input.trim()) {
            Ok(num) => return num,
            Err(_) => println!("Please enter a valid number."),
        }
    }
}

fn get_integer_input(prompt: &str) -> i32 {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        match input.trim().parse() {
            Ok(num) => return num,
            Err(_) => println!("Please enter a valid integer."),
        }
    }
}