use rust_decimal::Decimal;

pub fn secant_method<F>(
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

        // Avoid division by zero
        if f1 == f0 {
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
    use rust_decimal_macros::dec;

    #[test]
    fn test_secant_method_converges() {
        let f = |x: Decimal| x * x - dec!(2);
        let root = secant_method(f, dec!(1), dec!(2), dec!(0.0001), 100);
        assert!(root.is_some());
        let root = root.unwrap();
        assert!((root - dec!(1.4142)).abs() < dec!(0.0001));
    }

    #[test]
    fn test_secant_method_no_convergence() {
        let f = |x: Decimal| x * x + dec!(1);
        let root = secant_method(f, dec!(1), dec!(0), dec!(0.0001), 100);
        assert!(root.unwrap() == dec!(1));
    }

    #[test]
    fn test_secant_method_zero_derivative() {
        let f = |x: Decimal| x * x;
        let root = secant_method(f, dec!(1), dec!(1), dec!(0.0001), 100);
        assert!(root.unwrap() == dec!(1));
    }

    #[test]
    fn test_secant_method_linear_function() {
        let f = |x: Decimal| x - dec!(5);
        let root = secant_method(f, dec!(0), dec!(10), dec!(0.0001), 100);
        assert!(root.is_some());
        let root = root.unwrap();
        assert!((root - dec!(5)).abs() < dec!(0.0001));
    }

    #[test]
    fn test_secant_method_high_precision() {
        let f = |x: Decimal| x * x - dec!(2);
        let root = secant_method(f, dec!(1), dec!(2), dec!(0.00000001), 1000);
        assert!(root.is_some());
        let root = root.unwrap();
        assert!((root - dec!(1.41421356)).abs() < dec!(0.00000001));
    }
}
