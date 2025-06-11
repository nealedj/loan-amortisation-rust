use chrono::NaiveDate;
use clap::{Arg, Command};
use rust_decimal::prelude::*;
use serde_json::json;
use std::str::FromStr;

use loan_amortisation_rust::amortise::{amortise, InterestMethod, InterestType, Payment};

fn main() {
    let matches = parse_arguments();

    let principal = Decimal::from_str(matches.get_one::<String>("principal").unwrap()).unwrap();
    let annual_rate = Decimal::from_str(matches.get_one::<String>("annual_rate").unwrap()).unwrap()
        / Decimal::from(100);
    let num_payments = matches
        .get_one::<String>("num_payments")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let disbursal_date = NaiveDate::parse_from_str(
        matches.get_one::<String>("disbursal_date").unwrap(),
        "%Y-%m-%d",
    )
    .unwrap();
    let first_payment_date = NaiveDate::parse_from_str(
        matches.get_one::<String>("first_payment_date").unwrap(),
        "%Y-%m-%d",
    )
    .unwrap();
    let first_capitalisation_date = NaiveDate::parse_from_str(
        matches
            .get_one::<String>("first_capitalisation_date")
            .unwrap(),
        "%Y-%m-%d",
    )
    .unwrap();

    let interest_method = InterestMethod::from_str(
        matches
            .get_one::<String>("interest_method")
            .unwrap()
            .as_str(),
    )
    .unwrap();

    let interest_type = InterestType::from_str(
        matches
            .get_one::<String>("interest_type")
            .unwrap()
            .as_str(),
    )
    .unwrap();

    let output_format = matches.get_one::<String>("output_format").unwrap().as_str();

    let fixed_payment = matches
        .get_one::<String>("fixed_payment")
        .map(|fp| Decimal::from_str(fp).unwrap());

    let schedule = amortise(
        principal,
        annual_rate,
        num_payments,
        disbursal_date,
        first_payment_date,
        first_capitalisation_date,
        interest_method,
        interest_type,
        fixed_payment,
    );
    let payments = schedule.payments;

    match output_format {
        "json" => print_json(&payments),
        "tsv" => print_tsv(&payments),
        _ => print_table(&payments),
    }
}

fn parse_arguments() -> clap::ArgMatches {
    let matches = Command::new("Loan Amortisation Schedule Calculator")
        .version("1.0")
        .author("David Neale <david@neale.dev>")
        .about("Calculates loan amortisation schedules")
        .arg(Arg::new("principal")
            .short('p')
            .long("principal")
            .value_name("PRINCIPAL")
            .help("Sets the principal amount")
            .required(true))
        .arg(Arg::new("annual_rate")
            .short('r')
            .long("rate")
            .value_name("ANNUAL_RATE")
            .help("Sets the annual interest rate")
            .required(true))
        .arg(Arg::new("num_payments")
            .short('n')
            .long("num_payments")
            .value_name("NUM_PAYMENTS")
            .help("Sets the number of payments")
            .required(true))
        .arg(Arg::new("disbursal_date")
            .short('d')
            .long("disbursal_date")
            .value_name("DISBURSAL_DATE")
            .help("Sets the disbursal date (YYYY-MM-DD)")
            .required(true))
        .arg(Arg::new("first_payment_date")
            .short('f')
            .long("first_payment_date")
            .value_name("FIRST_PAYMENT_DATE")
            .help("Sets the first payment date (YYYY-MM-DD)")
            .required(true))
        .arg(Arg::new("first_capitalisation_date")
            .short('c')
            .long("first_capitalisation_date")
            .value_name("FIRST_CAPITALISATION_DATE")
            .help("Sets the first capitalisation date (YYYY-MM-DD)")
            .required(true))
        .arg(Arg::new("interest_method")
            .short('i')
            .long("interest_method")
            .default_value("ACTUALACTUAL")
            .value_name("INTEREST_METHOD")
            .help("Sets the interest method (Convention30_360, Actual365, Actual360, ActualActual)")
            .required(true))
        .arg(Arg::new("interest_type")
            .short('t')
            .long("interest_type")
            .default_value("Simple")
            .value_name("INTEREST_TYPE")
            .help("Sets the interest type (Simple, Compound)")
            .required(false))
        .arg(Arg::new("output_format")
            .short('o')
            .long("output_format")
            .default_value("table")
            .value_name("OUTPUT_FORMAT")
            .help("Sets the output format (json, table, tsv)")
            .required(false))
        .arg(Arg::new("fixed_payment")
            .long("fixed_payment")
            .value_name("FIXED_PAYMENT")
            .help("Sets a fixed monthly payment amount (optional)")
            .required(false))
        .get_matches();

    matches
}

fn print_row(
    month: u32,
    payment: Decimal,
    principal: Decimal,
    interest: Decimal,
    balance: Decimal,
) {
    println!(
        "{:5} | {:7.2} | {:9.2} | {:8.2} | {:17.2}",
        month, payment, principal, interest, balance
    );
}

fn print_table(schedule: &[Payment]) {
    println!("\nAmortisation Schedule:");
    println!("Month | Payment | Principal | Interest | Remaining Balance");
    for payment in schedule {
        print_row(
            payment.month,
            payment.payment,
            payment.principal,
            payment.interest,
            payment.balance,
        );
    }
}

fn print_json(schedule: &[Payment]) {
    let json_schedule: Vec<_> = schedule
        .iter()
        .map(|p| {
            json!({
                "month": p.month,
                "payment": p.payment,
                "principal": p.principal,
                "interest": p.interest,
                "balance": p.balance,
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&json_schedule).unwrap());
}

fn print_tsv(schedule: &[Payment]) {
    println!("Month\tPayment\tPrincipal\tInterest\tRemaining Balance");
    for payment in schedule {
        println!(
            "{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
            payment.month, payment.payment, payment.principal, payment.interest, payment.balance
        );
    }
}
