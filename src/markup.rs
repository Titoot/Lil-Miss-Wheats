const TTS_PREFIX: [u8; 12] = [0x0E, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x00];
const SEP_MARKER: [u8; 7] = [0x0E, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00];

fn pua_button_from_cp(cp: u32) -> Option<&'static str> {
    match cp {
        0xE0AB => Some("[A_ALT]"),
        0xE0AC => Some("[B_ALT]"),
        0xE0E0 => Some("[A]"),
        0xE0E1 => Some("[B]"),
        0xE0E3 => Some("[Y]"),
        0xE0E8 => Some("[SL]"),
        0xE0E9 => Some("[SR]"),
        0xE0EB => Some("[UP]"),
        0xE0EC => Some("[DOWN]"),
        0xE0ED => Some("[LEFT]"),
        0xE0EE => Some("[RIGHT]"),
        0xE0F0 => Some("[MINUS]"),
        0xE0F1 => Some("[+]"),
        _ => None,
    }
}

fn cp_from_pua_button(name: &str) -> Option<u32> {
    match name {
        "A" => Some(0xE0E0),
        "A_ALT" => Some(0xE0AB),
        "B" => Some(0xE0E1),
        "B_ALT" => Some(0xE0AC),
        "Y" => Some(0xE0E3),
        "SL" => Some(0xE0E8),
        "SR" => Some(0xE0E9),
        "UP" => Some(0xE0EB),
        "DOWN" => Some(0xE0EC),
        "LEFT" => Some(0xE0ED),
        "RIGHT" => Some(0xE0EE),
        "MINUS" => Some(0xE0F0),
        "+" => Some(0xE0F1),
        _ => None,
    }
}

pub fn binary_to_tagged(raw: &[u8]) -> String {
    if raw.is_empty() {
        return String::new();
    }

    let mut data = raw.to_vec();
    while data.last() == Some(&0x00) {
        data.pop();
    }

    let mut result = String::new();
    let mut pos = 0;

    while pos < data.len() {
        if pos + TTS_PREFIX.len() <= data.len() && data[pos..pos + TTS_PREFIX.len()] == TTS_PREFIX {
            let after_prefix = pos + TTS_PREFIX.len();

            if after_prefix + 6 > data.len() {
                result.push_str(&decode_text(&data[pos..]));
                break;
            }

            let _v1 = u16::from_le_bytes([data[after_prefix], data[after_prefix + 1]]);
            let v2 = u16::from_le_bytes([data[after_prefix + 2], data[after_prefix + 3]]);
            let v3 = u16::from_le_bytes([data[after_prefix + 4], data[after_prefix + 5]]);

            let tts_start = after_prefix + 6;
            let tts_end = tts_start + v3 as usize;
            let display_end = tts_end + v2 as usize;

            if tts_end > data.len() {
                result.push_str(&decode_text(&data[pos..]));
                break;
            }

            let tts_bytes = &data[tts_start..tts_end];
            let tts_text = decode_text(tts_bytes);

            let display_v2 = if display_end <= data.len() {
                decode_text(&data[tts_end..display_end])
            } else {
                String::new()
            };

            result.push_str(&format!("[TTS(\"{}\")]", tts_text));
            result.push_str(&display_v2);
            result.push_str("[-]");

            pos = display_end;
            continue;
        }
        else if pos + SEP_MARKER.len() <= data.len() && data[pos..pos + SEP_MARKER.len()] == SEP_MARKER {
            result.push_str("[SEP]");
            pos += SEP_MARKER.len();
        }
        else if data[pos] == 0x00 {
            result.push_str("[N]");
            pos += 1;
        }
        else {
            let start = pos;
            while pos < data.len() {
                if data[pos] == 0x00 {
                    break;
                }
                if pos + TTS_PREFIX.len() <= data.len() && data[pos..pos + TTS_PREFIX.len()] == TTS_PREFIX {
                    break;
                }
                if pos + SEP_MARKER.len() <= data.len() && data[pos..pos + SEP_MARKER.len()] == SEP_MARKER {
                    break;
                }
                pos += 1;
            }
            if pos > start {
                result.push_str(&decode_text(&data[start..pos]));
            }
        }
    }

    result
}

pub fn tagged_to_binary(tagged: &str, encoding: u8) -> Vec<u8> {
    let mut result = Vec::new();
    let chars: Vec<char> = tagged.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 6 < chars.len()
            && chars[i] == '['
            && chars[i + 1] == 'T'
            && chars[i + 2] == 'T'
            && chars[i + 3] == 'S'
            && chars[i + 4] == '('
            && chars[i + 5] == '"'
        {
            let mut j = i + 6;
            let mut tts_text = String::new();
            while j < chars.len() {
                if chars[j] == '"' && j + 1 < chars.len() && chars[j + 1] == ')' {
                    break;
                }
                tts_text.push(chars[j]);
                j += 1;
            }
            j += 2; // skip ")
            if j < chars.len() && chars[j] == ']' {
                j += 1;
            }

            let mut display_text = String::new();
            while j < chars.len() {
                if j + 2 < chars.len() && chars[j] == '[' && chars[j + 1] == '-' && chars[j + 2] == ']' {
                    j += 3;
                    break;
                }
                if j + 4 < chars.len() && chars[j] == '[' && chars[j + 1] == 'S' && chars[j + 2] == 'E' && chars[j + 3] == 'P' {
                    break;
                }
                if j + 5 < chars.len() && chars[j] == '[' && chars[j + 1] == 'T' && chars[j + 2] == 'T' && chars[j + 3] == 'S' {
                    break;
                }
                display_text.push(chars[j]);
                j += 1;
            }

            let tts_bytes = encode_text(&replace_button_tags(&tts_text), encoding);
            let display_bytes = encode_text(&replace_button_tags(&display_text), encoding);

            let v3 = tts_bytes.len() as u16;
            let v2 = display_bytes.len() as u16;
            let v1 = v3 + 4;

            result.extend_from_slice(&TTS_PREFIX);
            result.extend_from_slice(&v1.to_le_bytes());
            result.extend_from_slice(&v2.to_le_bytes());
            result.extend_from_slice(&v3.to_le_bytes());
            result.extend_from_slice(&tts_bytes);
            result.extend_from_slice(&display_bytes);

            i = j;
        }
        else if i + 4 < chars.len()
            && chars[i] == '['
            && chars[i + 1] == 'S'
            && chars[i + 2] == 'E'
            && chars[i + 3] == 'P'
            && chars[i + 4] == ']'
        {
            result.extend_from_slice(&SEP_MARKER);
            i += 5;
        }
        else if i + 2 < chars.len()
            && chars[i] == '['
            && chars[i + 1] == '-'
            && chars[i + 2] == ']'
        {
            i += 3;
        }
        else if i + 2 < chars.len()
            && chars[i] == '['
            && chars[i + 1] == 'N'
            && chars[i + 2] == ']'
        {
            result.push(0x00);
            i += 3;
        }
        else if chars[i] == '[' {
            let close = i + 1;
            let mut end = close;
            while end < chars.len() && chars[end] != ']' { end += 1; }
            if end < chars.len() && chars[end] == ']' {
                let tag: String = chars[close..end].iter().collect();
                if let Some(cp) = cp_from_pua_button(&tag) {
                    if encoding == 1 {
                        result.extend_from_slice(&(cp as u16).to_le_bytes());
                    } else {
                        let c = char::from_u32(cp).unwrap();
                        let mut buf = [0u8; 4];
                        let len = c.encode_utf8(&mut buf).len();
                        result.extend_from_slice(&buf[..len]);
                    }
                    i = end + 1;
                    continue;
                }
            }
            i += 1;
            result.extend_from_slice(&encode_text(&chars[i - 1].to_string(), encoding));
        }
        else {
            let start = i;
            while i < chars.len() {
                if chars[i] == '[' {
                    break;
                }
                i += 1;
            }
            if i == start {
                // Would infinite-loop: incomplete tag at current position, treat '[' as text
                i += 1;
                result.extend_from_slice(&encode_text(&chars[start].to_string(), encoding));
                continue;
            }
            let text: String = chars[start..i].iter().collect();
            result.extend_from_slice(&encode_text(&text, encoding));
        }
    }

    result.push(0x00);
    if encoding == 1 {
        result.push(0x00);
    }

    result
}

fn decode_text(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes).to_string();
    let mut result = String::new();
    for c in s.chars() {
        let cp = c as u32;
        if let Some(tag) = pua_button_from_cp(cp) {
            result.push_str(tag);
        } else {
            result.push(c);
        }
    }
    result
}

fn replace_button_tags(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' {
            let mut end = i + 1;
            while end < chars.len() && chars[end] != ']' {
                end += 1;
            }
            if end < chars.len() && chars[end] == ']' {
                let tag: String = chars[i + 1..end].iter().collect();
                if let Some(cp) = cp_from_pua_button(&tag) {
                    if let Some(c) = char::from_u32(cp) {
                        result.push(c);
                        i = end + 1;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn encode_text(text: &str, encoding: u8) -> Vec<u8> {
    if encoding == 1 {
        let mut bytes = Vec::new();
        for c in text.chars() {
            let mut buf = [0u16; 1];
            let _ = c.encode_utf16(&mut buf);
            bytes.extend_from_slice(&buf[0].to_le_bytes());
        }
        bytes
    } else {
        text.as_bytes().to_vec()
    }
}
