pub mod proof;
pub mod report;
pub mod key_management;
pub mod status;
pub mod double_echo;

pub const DIFICULTY : u128 = u128::max_value() - u128::max_value() / 10; // Increase to 500_000 for a real aplication, Average 500k hashes