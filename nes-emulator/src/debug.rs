use std::io::{self, Write};
pub fn print_bytes_hex(data: &[u8], address_offset: u16, line_width: usize) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for (i, chunk) in data.chunks(line_width).enumerate() {
        let _ = write!(handle, "{:04x}: ", address_offset as usize + i * line_width);
        for x in chunk {
            let _ = write!(handle, "{:02x}  ", x);
        }
        let _ = writeln!(handle, "");
    }
}
