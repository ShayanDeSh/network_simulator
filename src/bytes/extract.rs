pub fn extract_str(data: &[u8], start: usize, end: usize) -> &str {
    std::str::from_utf8(&data[start..end]).expect("Could not extract str")
}

pub fn extract_u16(data: &[u8], start: usize) -> u16 {
    ((data[start] as u16) << 8) + data[start + 1] as u16 
}

pub fn extract_ip(data: &[u8], start: usize) -> String {
    format!("{}.{}.{}.{}", data[start], data[start + 1],
        data[start + 2], data[start + 3])
}
