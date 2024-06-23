# equitycalc

Model equity and generate spreadsheets.

> [!CAUTION]
> This project is under active development and is highly unstable. Use at your own risk.

## Getting Started
1. Specify a portfolio (see below)
1. `cargo run total-report`

## Portfolio Specification

A portfolio consists of:

1. A `psp.yaml` file describing the preferred stock price over time. Example:
    ```yaml
    date: 2020-01-01
    price: 1.00
    ---
    date: 2021-01-01
    price: 1.75
    ---
    ...
    ```
1. An `option_grants.yaml` file describing the options grants recieved. Example:
    ```yaml
    name: New Hire
    date: 2020-01-01
    grant_value:
      exercise_price: 1.00
      shares: 1000
    vesting_schedule:
      commences_on: 2020-01-01
      events:
      - date: 2021-01-01
        number_of_shares: 250
      ...
    ---
    ...
    ```
1. An `rsu_grants.yaml` file describing the RSU grants recieved: Example:
    ```yaml
    name: 2020 Performance Grant
    date: 2021-01-01
    grant_value:
      grant_price: 1.75
      total_value: 1750
    vesting_schedule:
    commences_on: 2021-01-01
    events:
      - date: 2022-04-01
        number: 109
    ---
    ...
    ```

## TODO
- [x] RSUs
- [x] PSP
- [x] Options
- [x] Recognize vesting commencement date
- [x] Use a library for processing commands
- [x] Generate a csv of vesting amounts over time, by quarter
- [x] Parse portfolio models from YAML
- [ ] Generate a spreadsheet that is already formatted
