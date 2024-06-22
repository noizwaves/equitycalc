pub mod incr;
pub mod total;

fn format_currency(cents: i32) -> String {
    format!("{0}.{1:02}", cents / 100, cents % 100)
}
