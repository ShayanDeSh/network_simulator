pub fn copy_str(buf: &mut [u8], current: u16, string: &str) {
    let string = string.as_bytes();
    for (i, byte) in string.iter().enumerate() {
        buf[current as usize + i] = *byte;
    }
}

pub fn copy_u16(buf: &mut [u8], current: u16, num: u16) {
    let num = num.to_be_bytes();
    buf[current as usize] = num[0];
    buf[current as usize + 1] = num[1];
}

pub fn copy_ip(buf: &mut [u8], current: u16, ip: &str) {
    let ip: Vec<&str> = ip.split(".").collect();
    for (i, num) in ip.iter().enumerate() {
        buf[current as usize + i] = num.parse::<u8>().expect("Wrong ip");
    }
}
