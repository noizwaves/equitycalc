use clap::Parser;

mod dto;
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

fn run_command(command: Command, portfolio: PortfolioContext) {
    match command {
        Command::TotalReport => {
            let valuation = report::total::Valuation::new(
                &portfolio.psp,
                &portfolio.option_grants,
                &portfolio.rsu_grants,
            );

            valuation.print_to_csv();
        }
        Command::IncrementalReport(args) => {
            let report = report::incr::Report::new(
                &portfolio.psp,
                &portfolio.option_grants,
                &portfolio.rsu_grants,
                args.to_report_options(),
            );

            report.print_to_csv();
        }
    }
}

struct PortfolioContext {
    psp: model::psp::PreferredStockPrice,
    option_grants: Vec<model::option::OptionGrant>,
    rsu_grants: Vec<model::rsu::RestrictedStockUnitGrant>,
}

fn load_portfolio() -> PortfolioContext {
    let psp = dto::load_psp();
    let option_grants = dto::load_option_grants();
    let rsu_grants = portfolio::restricted_stock_grants();

    PortfolioContext {
        psp,
        option_grants,
        rsu_grants,
    }
}

fn main() {
    let args = Cli::parse();

    let portfolio = load_portfolio();

    run_command(args.command, portfolio);
}
