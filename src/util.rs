pub fn clamp<T: std::cmp::PartialOrd>(x: T, min: T, max: T) -> T {
    if x > max {
        return max;
    }
    if x < min {
        return min;
    }
    return x;
}
