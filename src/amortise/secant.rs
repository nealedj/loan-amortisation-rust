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
            return None;
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
