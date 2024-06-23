# equitycalc

Model equity and generate spreadsheets.

> [!CAUTION]
> This project is under active development and is highly unstable. Use at your own risk.

## Getting Started
1. Implement a Portfolio in `src/portfolio.rs` and specify a portfolio.
1. `cargo run`

## Portfolio Specification

A portfolio consists of:

1. A `psp.yaml` file describing the preferred stock price over time. Example:
    ```yaml
    ---
    date: 2020-01-01
    price: 1.00
    ---
    date: 2021-01-01
    price: 1.75
    ---
    ```

## TODO
- [x] RSUs
- [x] PSP
- [x] Options
- [x] Recognize vesting commencement date
- [x] Use a library for processing commands
- [x] Generate a csv of vesting amounts over time, by quarter
- [ ] Parse portfolio models from YAML
- [ ] Generate a spreadsheet that is already formatted
