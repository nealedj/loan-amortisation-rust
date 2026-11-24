use chrono::NaiveDate;
use clap::{Arg, Command};
use rust_decimal::prelude::*;
use std::str::FromStr;

use loan_amortisation_rust::amortise::{
    amortise, InterestMethod, InterestType, Payment, PaymentType,
};

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

    let balloon_payment = matches
        .get_one::<String>("balloon_payment")
        .map(|bp| Decimal::from_str(bp).unwrap());

    let option_fee = matches
        .get_one::<String>("option_fee")
        .map(|of| Decimal::from_str(of).unwrap());

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
        balloon_payment,
        option_fee,
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
        .arg(Arg::new("balloon_payment")
            .long("balloon_payment")
            .value_name("BALLOON_PAYMENT")
            .help("Sets a balloon payment amount due at the end (optional, for PCP loans)")
            .required(false))
        .arg(Arg::new("option_fee")
            .long("option_fee")
            .value_name("OPTION_FEE")
            .help("Sets an option fee amount for HP loans (optional)")
            .required(false))
        .get_matches();

    matches
}

fn payment_label(payment: &Payment) -> String {
    match payment.payment_type {
        PaymentType::Regular => payment.month.to_string(),
        PaymentType::Balloon => format!("{} (balloon)", payment.month),
    }
}

fn print_table(schedule: &[Payment]) {
    println!("\nAmortisation Schedule:");
    println!("Month        | Date       | Payment  | Principal | Interest | Fee     | Remaining Balance");
    for payment in schedule {
        println!(
            "{:12} | {} | {:8.2} | {:9.2} | {:8.2} | {:7.2} | {:17.2}",
            payment_label(payment),
            payment.date,
            payment.payment,
            payment.principal,
            payment.interest,
            payment.fee,
            payment.balance
        );
    }
}

fn print_json(schedule: &[Payment]) {
    println!("{}", serde_json::to_string_pretty(schedule).unwrap());
}

fn print_tsv(schedule: &[Payment]) {
    println!("Month\tDate\tPayment\tPrincipal\tInterest\tFee\tRemaining Balance");
    for payment in schedule {
        println!(
            "{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
            payment_label(payment),
            payment.date,
            payment.payment,
            payment.principal,
            payment.interest,
            payment.fee,
            payment.balance
        );
    }
}
