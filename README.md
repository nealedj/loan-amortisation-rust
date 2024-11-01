# Loan Amortisation Schedule Calculator

This Rust application calculates loan amortisation schedules. It takes several parameters as command line arguments and outputs a detailed amortisation schedule.

## Features

- Calculates monthly payments for a loan
- Provides a detailed amortisation schedule
- Supports different interest calculation methods

## Usage

To use this application, you need to provide the following command line arguments:

- `--principal` or `-p`: The principal amount of the loan
- `--rate` or `-r`: The annual interest rate (as a percentage)
- `--num_payments` or `-n`: The number of payments (months)
- `--disbursal_date` or `-d`: The disbursal date of the loan (YYYY-MM-DD)
- `--first_payment_date` or `-f`: The date of the first payment (YYYY-MM-DD)
- `--first_capitalisation_date` or `-c`: The first capitalisation date (YYYY-MM-DD)
- `--interest_method` or `-i`: The interest calculation method (Convention30_360, Actual365, Actual360, ActualActual)

### Example

```sh
cargo run -- \
    --principal 15000 \
    --rate 8.9 \
    --num_payments 36 \
    --disbursal_date 2023-01-01 \
    --first_payment_date 2023-02-01 \
    --first_capitalisation_date 2023-01-15 \
    --interest_method ActualActual
```

### Building
To build the executable, run the following command in the root directory of the project:

```sh
cargo build
```

For a release build, use:

```sh
cargo build --release
```

The executable will be located in the target/debug or target/release directory, respectively.

### Running Tests
To run the tests for this project, use the following command:

```sh
cargo test
```

### Dependencies
This project uses the following dependencies:

- `clap`: For parsing command line arguments
- `chrono`: For handling dates
- `rust_decimal`: For precise decimal arithmetic

### License
This project is licensed under the GNU General Public License v3.0. See the LICENSE file for details.

The GNU General Public License v3.0 ensures that this software remains free for all its users. You are free to use, modify, and distribute this software, provided that any derivative works are also licensed under the same terms. This license guarantees that users interacting with the software over a network can receive the source code of the software.

### Author
David Neale - david@neale.dev

