use std::collections::HashMap;

fn letters_arabic() -> HashMap<char, [char; 4]> {
    let mut m = HashMap::new();
    m.insert('ء', ['\u{FE80}', '\0', '\0', '\0']);
    m.insert('آ', ['\u{FE81}', '\0', '\0', '\u{FE82}']);
    m.insert('أ', ['\u{FE83}', '\0', '\0', '\u{FE84}']);
    m.insert('ؤ', ['\u{FE85}', '\0', '\0', '\u{FE86}']);
    m.insert('إ', ['\u{FE87}', '\0', '\0', '\u{FE88}']);
    m.insert('ئ', ['\u{FE89}', '\u{FE8B}', '\u{FE8C}', '\u{FE8A}']);
    m.insert('ا', ['\u{FE8D}', '\0', '\0', '\u{FE8E}']);
    m.insert('ب', ['\u{FE8F}', '\u{FE91}', '\u{FE92}', '\u{FE90}']);
    m.insert('ة', ['\u{FE93}', '\0', '\0', '\u{FE94}']);
    m.insert('ت', ['\u{FE95}', '\u{FE97}', '\u{FE98}', '\u{FE96}']);
    m.insert('ث', ['\u{FE99}', '\u{FE9B}', '\u{FE9C}', '\u{FE9A}']);
    m.insert('ج', ['\u{FE9D}', '\u{FE9F}', '\u{FEA0}', '\u{FE9E}']);
    m.insert('ح', ['\u{FEA1}', '\u{FEA3}', '\u{FEA4}', '\u{FEA2}']);
    m.insert('خ', ['\u{FEA5}', '\u{FEA7}', '\u{FEA8}', '\u{FEA6}']);
    m.insert('د', ['\u{FEA9}', '\0', '\0', '\u{FEAA}']);
    m.insert('ذ', ['\u{FEAB}', '\0', '\0', '\u{FEAC}']);
    m.insert('ر', ['\u{FEAD}', '\0', '\0', '\u{FEAE}']);
    m.insert('ز', ['\u{FEAF}', '\0', '\0', '\u{FEB0}']);
    m.insert('س', ['\u{FEB1}', '\u{FEB3}', '\u{FEB4}', '\u{FEB2}']);
    m.insert('ش', ['\u{FEB5}', '\u{FEB7}', '\u{FEB8}', '\u{FEB6}']);
    m.insert('ص', ['\u{FEB9}', '\u{FEBB}', '\u{FEBC}', '\u{FEBA}']);
    m.insert('ض', ['\u{FEBD}', '\u{FEBF}', '\u{FEC0}', '\u{FEBE}']);
    m.insert('ط', ['\u{FEC1}', '\u{FEC3}', '\u{FEC4}', '\u{FEC2}']);
    m.insert('ظ', ['\u{FEC5}', '\u{FEC7}', '\u{FEC8}', '\u{FEC6}']);
    m.insert('ع', ['\u{FEC9}', '\u{FECB}', '\u{FECC}', '\u{FECA}']);
    m.insert('غ', ['\u{FECD}', '\u{FECF}', '\u{FED0}', '\u{FECE}']);
    m.insert('ف', ['\u{FED1}', '\u{FED3}', '\u{FED4}', '\u{FED2}']);
    m.insert('ق', ['\u{FED5}', '\u{FED7}', '\u{FED8}', '\u{FED6}']);
    m.insert('ك', ['\u{FED9}', '\u{FEDB}', '\u{FEDC}', '\u{FEDA}']);
    m.insert('ل', ['\u{FEDD}', '\u{FEDF}', '\u{FEE0}', '\u{FEDE}']);
    m.insert('م', ['\u{FEE1}', '\u{FEE3}', '\u{FEE4}', '\u{FEE2}']);
    m.insert('ن', ['\u{FEE5}', '\u{FEE7}', '\u{FEE8}', '\u{FEE6}']);
    m.insert('ه', ['\u{FEE9}', '\u{FEEB}', '\u{FEEC}', '\u{FEEA}']);
    m.insert('و', ['\u{FEED}', '\0', '\0', '\u{FEEE}']);
    m.insert('ى', ['\u{FEEF}', '\0', '\0', '\u{FEF0}']);
    m.insert('ي', ['\u{FEF1}', '\u{FEF3}', '\u{FEF4}', '\u{FEF2}']);
    m.insert('پ', ['\u{FB56}', '\u{FB58}', '\u{FB59}', '\u{FB57}']);
    m.insert('ڤ', ['\u{FB6A}', '\u{FB6C}', '\u{FB6D}', '\u{FB6B}']);
    m
}

fn ligatures() -> HashMap<String, char> {
    let mut m = HashMap::new();
    m.insert(format!("\u{FEDF}\u{FE8E}"), '\u{FEFB}');
    m.insert(format!("\u{FEE0}\u{FE8E}"), '\u{FEFC}');
    m.insert(format!("\u{FEDF}\u{FE84}"), '\u{FEF7}');
    m.insert(format!("\u{FEE0}\u{FE84}"), '\u{FEF8}');
    m.insert(format!("\u{FEDF}\u{FE88}"), '\u{FEF9}');
    m.insert(format!("\u{FEE0}\u{FE88}"), '\u{FEFA}');
    m.insert(format!("\u{FEDF}\u{FE82}"), '\u{FEF5}');
    m.insert(format!("\u{FEE0}\u{FE82}"), '\u{FEF6}');
    m
}

fn connects_with_next(c: char, map: &HashMap<char, [char; 4]>) -> bool {
    map.get(&c).map_or(false, |forms| forms[1] != '\0')
}

fn connects_with_prev(c: char, map: &HashMap<char, [char; 4]>) -> bool {
    map.get(&c).map_or(false, |forms| forms[3] != '\0')
}

pub fn reshape_arabic(text: &str) -> String {
    let map = letters_arabic();
    let chars: Vec<char> = text.chars().collect();
    let mut result = Vec::new();

    for i in 0..chars.len() {
        let c = chars[i];
        let Some(forms) = map.get(&c) else {
            result.push(c);
            continue;
        };

        let prev = if i > 0 { chars[i - 1] } else { '\0' };
        let next = if i < chars.len() - 1 { chars[i + 1] } else { '\0' };

        let conn_prev = connects_with_prev(c, &map) && connects_with_next(prev, &map);
        let conn_next = connects_with_next(c, &map) && connects_with_prev(next, &map);

        let form_idx = match (conn_prev, conn_next) {
            (true, true) => 2,  // Medial
            (true, false) => 3, // Final
            (false, true) => 1, // Initial
            (false, false) => 0, // Isolated
        };

        let shaped = forms[form_idx];
        result.push(if shaped != '\0' { shaped } else { c });
    }

    let mut s: String = result.into_iter().collect();
    for (lig, replacement) in ligatures() {
        s = s.replace(&lig, &replacement.to_string());
    }
    s
}

fn is_arabic(c: char) -> bool {
    matches!(c,
        '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}'
        | '\u{08A0}'..='\u{08FF}' | '\u{FB50}'..='\u{FDFF}'
        | '\u{FE70}'..='\u{FEFF}'
    )
}

pub fn process_arabic(text: &str) -> String {
    if !text.chars().any(is_arabic) {
        return text.to_string();
    }
    let shaped = reshape_arabic(text);
    shaped.split_whitespace()
        .rev()
        .map(|word| {
            if word.chars().any(is_arabic) {
                word.chars().rev().collect::<String>()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
