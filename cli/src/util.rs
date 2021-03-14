pub fn parse_amount_with_currency(amount: &str, symbol: &str, precision: u32) -> Option<i64> {
    if amount.ends_with(symbol) {
        amount
            .replace(symbol, "")
            .replace("_", "")
            .parse::<i64>()
            .ok()
            .map(|num| num * (10_i64.pow(precision)))
    } else {
        amount.replace("_", "").parse().ok()
    }
}

pub fn parse_amount(amount: &str) -> Option<i64> {
    amount.replace("_", "").parse().ok()
}
