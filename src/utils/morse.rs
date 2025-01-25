/// Converts a digit to the corresponding Morse code.
///
/// # Arguments
/// * `digit` - A single digit (0-9).
///
/// # Returns
/// * A static string representing the Morse code for the digit.
fn digit_to_morse(digit: u8) -> &'static str {
    match digit {
        0 => "-----",
        1 => ".----",
        2 => "..---",
        3 => "...--",
        4 => "....-",
        5 => ".....",
        6 => "-....",
        7 => "--...",
        8 => "---..",
        9 => "----.",
        _ => unreachable!(), // Guaranteed by the range of `digit` (0-9).
    }
}

/// Converts a number into a Morse code string.
///
/// # Arguments
/// * `number` - The number to be converted (u16).
/// * `buffer` - A mutable buffer for writing the Morse code representation.
///
/// # Returns
/// * `Ok(usize)` - The length of the data written to the buffer.
/// * `Err(&'static str)` - An error if the buffer is too small.
///
/// # Example
/// ```
/// let mut buffer = [0u8; 64];
/// let length = number_to_morse(123, &mut buffer).unwrap();
/// assert_eq!(&buffer[..length], b".---- ..--- ...--");
/// ```
pub fn number_to_morse(number: u16, buffer: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufferWriter::new(buffer);
    let mut first = true;

    // Iterate over the digits of the number, starting with the most significant digit
    let mut divisor = 10000; // Maximum divisor for u16 (65535)
    while divisor > 0 {
        let digit = (number / divisor) % 10;
        if digit != 0 || !first || divisor == 1 {
            if !first {
                writer.write_byte(b' ')?; // Add a space between Morse symbols
            }
            writer.write_str(digit_to_morse(digit as u8))?;
            first = false;
        }
        divisor /= 10;
    }

    Ok(writer.index) // Return the length of the written data
}

/// Helper structure for writing to a buffer.
struct BufferWriter<'a> {
    buffer: &'a mut [u8],
    index: usize,
}

impl<'a> BufferWriter<'a> {
    /// Creates a new `BufferWriter`.
    fn new(buffer: &'a mut [u8]) -> Self {
        BufferWriter { buffer, index: 0 }
    }

    /// Writes a single byte to the buffer.
    ///
    /// # Arguments
    /// * `byte` - The byte to write.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully written.
    /// * `Err(&'static str)` - Error if the buffer overflows.
    fn write_byte(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.index >= self.buffer.len() {
            return Err("Buffer overflow");
        }
        self.buffer[self.index] = byte;
        self.index += 1;
        Ok(())
    }

    /// Writes a string to the buffer.
    ///
    /// # Arguments
    /// * `s` - The string to write.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully written.
    /// * `Err(&'static str)` - Error if the buffer overflows.
    fn write_str(&mut self, s: &str) -> Result<(), &'static str> {
        for byte in s.bytes() {
            self.write_byte(byte)?;
        }
        Ok(())
    }
}
