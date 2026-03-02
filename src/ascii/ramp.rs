pub const STANDARD_NAME: &str = "standard";
pub const DETAILED_NAME: &str = "detailed";

const STANDARD: &[u8] = b" .:-=+*#%@";
const DETAILED: &[u8] = b" .'`^\",:;Il!i~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";

pub fn is_valid(name: &str) -> bool {
    matches!(name, STANDARD_NAME | DETAILED_NAME)
}

pub fn by_name(name: &str) -> &'static [u8] {
    match name {
        DETAILED_NAME => DETAILED,
        _ => STANDARD,
    }
}

pub fn next_name(current: &str) -> &'static str {
    match current {
        STANDARD_NAME => DETAILED_NAME,
        _ => STANDARD_NAME,
    }
}
