pub mod incr;
pub mod total;

fn format_currency(cents: i32) -> String {
    format!("{0}.{1:02}", cents / 100, cents % 100)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_format_currency() {
        assert_eq!("0.00", super::format_currency(0));
        assert_eq!("0.01", super::format_currency(1));
        assert_eq!("0.10", super::format_currency(10));
        assert_eq!("1.00", super::format_currency(100));
    }
}
