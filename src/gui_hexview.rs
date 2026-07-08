use gtk4::prelude::*;

pub fn fill_hex_buffer(buf: &gtk4::TextBuffer, data: &[u8]) {
    buf.set_text("");

    if data.is_empty() {
        return;
    }

    // Format as hex dump: offset | 16 hex bytes | ASCII
    let mut lines: Vec<String> = Vec::new();
    let len = data.len();

    let mut i = 0;
    while i < len {
        let end = std::cmp::min(i + 16, len);
        let count = end - i;

        // Offset
        let mut line = format!("{:04X}  ", i);

        // Hex bytes
        for j in i..end {
            line.push_str(&format!("{:02X} ", data[j]));
        }
        // Pad missing bytes in the last line
        if count < 16 {
            for _ in 0..(16 - count) {
                line.push_str("   ");
            }
        }

        line.push_str(" |");

        // ASCII representation
        for j in i..end {
            let c = data[j];
            if c >= 0x20 && c <= 0x7E {
                line.push(c as char);
            } else {
                line.push('.');
            }
        }

        line.push('|');
        lines.push(line);
        i = end;
    }

    buf.set_text(&lines.join("\n"));
}
