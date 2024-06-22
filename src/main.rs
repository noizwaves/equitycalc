use clap::Parser;

mod model;
mod portfolio;
mod report;

use clap::Subcommand;

#[derive(Parser)]
#[command()]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate report of vested, unvested, and total equity value per day
    TotalReport,

    /// Generate report of the equity vesting per quarter
    IncrementalReport,
}

fn run_command(command: Command) {
    match command {
        Command::TotalReport => {
            let valuation = report::total::Valuation::new(
                &portfolio::preferred_stock_price(),
                &portfolio::option_grants(),
                &portfolio::restricted_stock_grants(),
            );

            valuation.print_to_csv();
        }
        Command::IncrementalReport => {
            let report = report::incr::Report::new(
                &portfolio::preferred_stock_price(),
                &portfolio::option_grants(),
                &portfolio::restricted_stock_grants(),
            );

            report.print_to_csv();
        }
    }
}

fn main() {
    let args = Cli::parse();

    run_command(args.command);
}
