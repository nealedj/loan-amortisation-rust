use chrono::{Days, Months, NaiveDate};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

const DEFAULT_SCALE: u32 = 2;
const DEFAULT_PRECISION: u32 = 28;
const DEFAULT_ROUNDING: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;
const PERIODS_PER_YEAR: u32 = 12;

#[derive(PartialEq, Copy, Clone)]
pub enum InterestMethod {
    Convention30_360,
    Actual365,
    Actual360,
    ActualActual,
}

#[derive(Debug)]
pub struct Payment {
    pub month: u32,
    pub payment: Decimal,
    pub principal: Decimal,
    pub interest: Decimal,
    pub balance: Decimal,
}


pub fn amortise(
    principal: Decimal,
    annual_rate: Decimal,
    num_payments: u32,
    disbursal_date: NaiveDate,
    first_payment_date: NaiveDate,
    first_capitalisation_date: NaiveDate,
    interest_method: InterestMethod,
) -> Vec<Payment> {

    // Convert basis points to period rate, scaled
    let daily_rate = get_daily_interest_rate(annual_rate, interest_method);

    let mut period_payment = calculate_rough_period_payment(principal, annual_rate, num_payments);

    let f = |period_payment| {
        let schedule = build_schedule(
            principal,
            disbursal_date,
            first_capitalisation_date,
            first_payment_date,
            num_payments,
            daily_rate,
            period_payment,
            interest_method,
            false,
        );
        schedule.last().unwrap().balance // final balance
    };

    match secant_method(
        f,
        period_payment,
        period_payment * Decimal::new(1, 2),
        Decimal::new(1, 6),
        100,
    ) {
        Some(root) => period_payment = root,
        None => println!("Failed to converge"),
    }

    period_payment = round_decimal(period_payment, None, None, None);
    let schedule = build_schedule(
        principal,
        disbursal_date,
        first_capitalisation_date,
        first_payment_date,
        num_payments,
        daily_rate,
        period_payment,
        interest_method,
        true,
    );

    schedule
}

fn build_schedule(principal: Decimal, disbursal_date: NaiveDate, first_capitalisation_date: NaiveDate, first_payment_date: NaiveDate, num_payments: u32, daily_rate: Decimal, period_payment: Decimal, interest_method: InterestMethod, settle_balance: bool) -> Vec<Payment> {
    let mut schedule = Vec::new();
    let mut balance = principal;
    let mut interest_payable_from = disbursal_date;
    let mut next_cap_date = first_capitalisation_date;
    let mut next_payment_date = first_payment_date;

    for month in 1..=num_payments {
        let interest = calculate_period_interest(interest_payable_from, next_cap_date, next_payment_date, daily_rate, balance, period_payment, interest_method);
        let principal_payment;
        
        if settle_balance && month == num_payments {
            principal_payment = balance;
        } else {
            principal_payment = round_decimal(period_payment - interest, None, None, None);
        }

        balance = round_decimal(balance-principal_payment, None, None, None);

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

fn get_daily_interest_rate(annual_rate: Decimal, interest_method: InterestMethod) -> Decimal {
    let daily_rate: Decimal;

    match interest_method {
        InterestMethod::Convention30_360 => {
            daily_rate = annual_rate / Decimal::from(360);
        }
        InterestMethod::Actual365 => {
            daily_rate = annual_rate / Decimal::from(365);
        }
        InterestMethod::Actual360 => {
            daily_rate = annual_rate / Decimal::from(360);
        }
        InterestMethod::ActualActual => {
            daily_rate = annual_rate / Decimal::from(365); // adjusted later for leap years
        }
    }

    daily_rate
}

fn calculate_period_interest(
    start_date: NaiveDate,
    to_date: NaiveDate,
    payment_date: NaiveDate,
    daily_rate: Decimal,
    balance: Decimal,
    payment_amount: Decimal,
    interest_method: InterestMethod,
) -> Decimal {
    if interest_method == InterestMethod::Convention30_360 {
        return Decimal::from(30) * balance * daily_rate;
    }

    let mut current_date = start_date;
    let mut interest = Decimal::from(0);

    let mut balance_m = balance;
    let mut daily_rate_m = daily_rate;
    while current_date <= to_date {
        if interest_method == InterestMethod::ActualActual && current_date.leap_year() {
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

fn round_decimal(
    value: Decimal,
    precision: Option<u32>,
    scale: Option<u32>,
    rounding: Option<RoundingStrategy>,
) -> Decimal {
    let precision = precision.unwrap_or(DEFAULT_PRECISION);
    let scale = scale.unwrap_or(DEFAULT_SCALE);
    let rounding = rounding.unwrap_or(DEFAULT_ROUNDING);
    value.round_dp_with_strategy(scale.min(precision), rounding)
}

fn secant_method<F>(
    f: F,
    x0: Decimal,
    x1: Decimal,
    epsilon: Decimal,
    max_iterations: usize,
) -> Option<Decimal>
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_schedule() {
        let principal = Decimal::from(15000);
        let annual_rate = Decimal::from_str("8.9").unwrap() / Decimal::from(100);

        let num_payments = 36;
        let period_payment = Decimal::from_str("474.5").unwrap();

        let disbursal_date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        let first_capitalisation_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let first_payment_date = NaiveDate::from_ymd_opt(2023, 3, 1).unwrap();
        let daily_rate = get_daily_interest_rate(annual_rate, InterestMethod::ActualActual);

        let schedule = build_schedule(
            principal,
            disbursal_date,
            first_capitalisation_date,
            first_payment_date,
            num_payments,
            daily_rate,
            period_payment,
            InterestMethod::ActualActual,
            true,
        );

        assert_eq!(schedule.len(), 36);
        assert_eq!(schedule.last().unwrap().balance, Decimal::from(0));
    }

    #[test]
    fn test_calculate_rough_period_payment() {
        let principal = Decimal::from(15000);
        let annual_rate = Decimal::from_f64(8.9).unwrap() / Decimal::from(100);
        let num_payments = 36;

        let period_payment = calculate_rough_period_payment(principal, annual_rate, num_payments);

        assert!(period_payment > Decimal::from(0));
    }

    #[test]
    fn test_get_daily_interest_rate() {
        let annual_rate = Decimal::from_f64(8.9).unwrap() / Decimal::from(100);
        let interest_method = InterestMethod::ActualActual;

        let daily_rate = get_daily_interest_rate(annual_rate, interest_method);

        assert!(daily_rate > Decimal::from(0));
    }
}
