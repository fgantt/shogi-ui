//! Common small helpers centralized for reuse.

/// Saturating clamp for i32.
#[inline]
pub fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_i32_bounds_and_middle() {
        assert_eq!(clamp_i32(-10, 0, 5), 0);
        assert_eq!(clamp_i32(10, 0, 5), 5);
        assert_eq!(clamp_i32(3, 0, 5), 3);
    }
}


