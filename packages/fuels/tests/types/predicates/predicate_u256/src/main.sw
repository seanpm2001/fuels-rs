predicate;

use std::u256::U256;

#[allow(deprecated)]
fn main(arg: U256) -> bool {
    arg == U256::from((10, 11, 12, 13))
}