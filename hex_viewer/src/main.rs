use std::{env, fs::File, io::Read};

const BYTES_PER_LINE: usize = 16;

fn main() -> std::io::Result<()> {
    let arg1 = env::args().nth(1);
    let fname = arg1.expect("Usage: hex_viewer filename");

    let mut f = File::open(fname).expect("Failed to open the file.");
    let mut pos = 0;
    let mut buffer = [0; BYTES_PER_LINE];

    while f.read_exact(&mut buffer).is_ok() {
        print!("[0x{:08x}] ", pos);
        for byte in &buffer {
            match *byte {
                0x00 => print!(".  "),
                0xff => print!("0xFF"),
                _ => print!("{:02x} ", byte),
            }
        }
        println!();
        pos += BYTES_PER_LINE;
    }

    Ok(())
}
