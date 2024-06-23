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
    IncrementalReport(IncrementalReportArgs),
}

#[derive(Debug, Parser, Default)]
struct IncrementalReportArgs {
    /// Use quarters of 12/17-3/16, 3/17-6/16, 6/17-9/16, and 9/17-12/16
    #[arg(long = "skewed", default_value = "false")]
    pub skewed_quarter_dates: bool,
}

impl IncrementalReportArgs {
    pub fn to_report_options(&self) -> report::incr::ReportOptions {
        report::incr::ReportOptions {
            quarter_type: if self.skewed_quarter_dates {
                report::incr::QuarterType::Skewed
            } else {
                report::incr::QuarterType::Calendar
            },
        }
    }
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
        Command::IncrementalReport(args) => {
            let report = report::incr::Report::new(
                &portfolio::preferred_stock_price(),
                &portfolio::option_grants(),
                &portfolio::restricted_stock_grants(),
                args.to_report_options(),
            );

            report.print_to_csv();
        }
    }
}

fn main() {
    let args = Cli::parse();

    run_command(args.command);
}
