use serialport::SerialPort;
use std::str;

pub fn read_line(mut port: Box<dyn SerialPort>) -> String {
    // NMEA Max message length = 82
    let mut buffer = [0; 82];
    loop {
        port.read(&mut buffer[..]).unwrap();

        // 10 = \n
        if buffer.ends_with(&[10]) {
            return String::from(str::from_utf8(&buffer).unwrap());
        }
    }
}
