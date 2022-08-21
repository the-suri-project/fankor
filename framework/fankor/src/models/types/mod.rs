pub mod integers;

pub mod maps;
pub mod sets;
pub mod strings;
pub mod unsigned;
pub mod vectors;

fn read_length(buf: &mut &[u8], size: usize) -> std::io::Result<u32> {
    if buf.len() < size {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Unexpected length of input",
        ));
    }

    let mut len = 0;
    let mut shift = 0;
    for i in 0..size {
        len |= (buf[i] as u32) << shift;
        shift += 8;
    }

    *buf = &buf[size..];

    Ok(len)
}
