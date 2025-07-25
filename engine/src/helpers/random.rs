// if using sqlite, we can use the following function to generate a random ID within the range of positive i64 values
/// Generate a random ID within the range of positive i64 values
pub fn generate_random_id() -> i64 {
    rand::random::<i64>() & i64::MAX // Clear sign bit
}
