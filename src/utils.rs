pub fn truncate_after_decimal(value: &str, places: usize) -> String {
    let parts: Vec<&str> = value.split('.').collect();
    match parts.len() {
        1 => value.to_string(), // 소수점이 없는 경우
        2 => {
            let integer_part = parts[0];
            let fractional_part = &parts[1][..places.min(parts[1].len())];
            if fractional_part.is_empty() {
                integer_part.to_string()
            } else {
                format!("{}.{}", integer_part, fractional_part)
            }
        }
        _ => value.to_string(), // 잘못된 형식인 경우 원본 반환
    }
}
