pub fn print_bytes_hex(data: &[u8], line_width: usize) {
    for (i, chunk) in data.chunks(line_width).enumerate() {
        print!("{:08}: ", i * line_width);
        for x in chunk {
            print!("{:02x}  ", x);
        }
        println!("");
    }
}
