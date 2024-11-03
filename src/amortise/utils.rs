use rust_decimal::Decimal;
use rust_decimal::prelude::RoundingStrategy;
const DEFAULT_SCALE: u32 = 2;
const DEFAULT_PRECISION: u32 = 28;
const DEFAULT_ROUNDING: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;

pub fn round_decimal(
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
