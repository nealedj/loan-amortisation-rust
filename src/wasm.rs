use crate::amortise::{amortise, InterestMethod, InterestType};
use chrono::NaiveDate;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde_wasm_bindgen::to_value;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn amortise_wasm(
    principal: f64,
    annual_rate: f64,
    num_payments: u32,
    disbursal_date: String,
    first_payment_date: String,
    first_capitalisation_date: String,
    interest_method: String,
    interest_type: String,
    fixed_payment: Option<f64>,
) -> JsValue {
    let principal = Decimal::from_f64(principal).unwrap();
    let annual_rate = Decimal::from_f64(annual_rate).unwrap() / Decimal::from(100);

    let disbursal_date = NaiveDate::parse_from_str(&disbursal_date, "%Y-%m-%d").unwrap();
    let first_payment_date = NaiveDate::parse_from_str(&first_payment_date, "%Y-%m-%d").unwrap();
    let first_capitalisation_date =
        NaiveDate::parse_from_str(&first_capitalisation_date, "%Y-%m-%d").unwrap();
    let interest_method = InterestMethod::from_str(&interest_method).unwrap();
    let interest_type = InterestType::from_str(&interest_type).unwrap();

    let fixed_payment_decimal = fixed_payment.map(|fp| Decimal::from_f64(fp).unwrap());

    let schedule = amortise(
        principal,
        annual_rate,
        num_payments,
        disbursal_date,
        first_payment_date,
        first_capitalisation_date,
        interest_method,
        interest_type,
        fixed_payment_decimal,
    );
    to_value(&schedule).unwrap()
}
