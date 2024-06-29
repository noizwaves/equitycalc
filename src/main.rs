use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

mod dto;
mod model;
mod report;

use clap::Subcommand;

#[derive(Parser)]
#[command()]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    /// Location of portfolio files
    #[arg(long = "portfolio-path", default_value = ".")]
    portfolio_path: String,
}

#[derive(Subcommand)]
enum Command {
    /// Generate report of vested, unvested, and total equity value per day
    TotalReport(TotalReportArgs),

    /// Generate report of the equity vesting per quarter
    IncrementalReport(IncrementalReportArgs),
}

#[derive(Parser)]
struct TotalReportArgs {
    /// Destination file name
    #[arg(long = "destination", default_value = "total.csv")]
    pub destination: String,
}

#[derive(Debug, Parser, Default)]
struct IncrementalReportArgs {
    /// Use quarters of 12/17-3/16, 3/17-6/16, 6/17-9/16, and 9/17-12/16
    #[arg(long = "skewed", default_value = "false")]
    pub skewed_quarter_dates: bool,

    /// Destination file name
    #[arg(long = "destination", default_value = "incremental.csv")]
    pub destination: String,
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
        Command::TotalReport(args) => {
            let destination = PathBuf::from(&args.destination);

            let valuation = report::total::Valuation::new(
                &portfolio.psp,
                &portfolio.option_grants,
                &portfolio.rsu_grants,
            );

            valuation.print_to_file(&destination);

            println!("Wrote total report to {:?}", destination)
        }
        Command::IncrementalReport(args) => {
            let destination = PathBuf::from(&args.destination);

            let report = report::incr::Report::new(
                &portfolio.psp,
                &portfolio.option_grants,
                &portfolio.rsu_grants,
                args.to_report_options(),
            );

            report.print_to_file(&destination);

            println!("Wrote incremental report to {:?}", destination);
        }
    }
}

struct PortfolioContext {
    psp: model::psp::PreferredStockPrice,
    option_grants: Vec<model::option::OptionGrant>,
    rsu_grants: Vec<model::rsu::RestrictedStockUnitGrant>,
}

fn load_portfolio(path: &String) -> PortfolioContext {
    let path = Path::new(path);
    let psp = dto::load_psp(path);
    let option_grants = dto::load_option_grants(path);
    let rsu_grants = dto::load_rsu_grants(path);

    PortfolioContext {
        psp,
        option_grants,
        rsu_grants,
    }
}

fn main() {
    let args = Cli::parse();

    let portfolio = load_portfolio(&args.portfolio_path);

    run_command(args.command, portfolio);
}
