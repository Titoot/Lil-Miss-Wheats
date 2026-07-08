use std::path::Path;
use std::fs;
use std::io;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::markup;

    #[test]
    fn roundtrip_reading_name() {
        let path = PathBuf::from("test_data/reading_name.msbt");
        let original = fs::read(&path).unwrap();
        let msbt = parse_msbt(&path).unwrap();

        // Rebuild strings in TXT2 index order
        let num_strings = msbt.num_original_strings;
        let mut strings = vec![Vec::new(); num_strings];
        for entry in &msbt.entries {
            if entry.index < num_strings {
                strings[entry.index] = entry.raw_bytes.clone();
            }
        }

        let tmp = PathBuf::from("_test_rd1.msbt");
        write_msbt(&tmp, &msbt, &strings).unwrap();

        let saved = fs::read(&tmp).unwrap();
        assert_eq!(original.len(), saved.len(), "File sizes differ");
        assert_eq!(original, saved, "File contents differ");
        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn roundtrip_stage_parasol() {
        let path = PathBuf::from("test_data/stage_parasol_00.msbt");
        let original = fs::read(&path).unwrap();
        let msbt = parse_msbt(&path).unwrap();

        let num_strings = msbt.num_original_strings;
        let mut strings = vec![Vec::new(); num_strings];
        for entry in &msbt.entries {
            if entry.index < num_strings {
                strings[entry.index] = entry.raw_bytes.clone();
            }
        }

        let tmp = PathBuf::from("_test_rd2.msbt");
        write_msbt(&tmp, &msbt, &strings).unwrap();

        let saved = fs::read(&tmp).unwrap();
        assert_eq!(original.len(), saved.len(), "File sizes differ for stage_parasol");
        assert_eq!(original, saved, "File contents differ for stage_parasol");
        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn roundtrip_reading_name_via_markup() {
        let path = PathBuf::from("test_data/reading_name.msbt");
        let original = fs::read(&path).unwrap();
        let msbt = parse_msbt(&path).unwrap();

        let encoding = msbt.header.encoding;
        let num_strings = msbt.num_original_strings;

        let mut strings = vec![Vec::new(); num_strings];
        for entry in &msbt.entries {
            let tagged = markup::binary_to_tagged(&entry.raw_bytes);
            let binary = markup::tagged_to_binary(&tagged, encoding);
            assert_eq!(
                entry.raw_bytes, binary,
                "Markup roundtrip mismatch for label='{}' index={}",
                entry.label, entry.index
            );
            strings[entry.index] = binary;
        }

        let tmp = PathBuf::from("_test_rd1_markup.msbt");
        write_msbt(&tmp, &msbt, &strings).unwrap();
        let saved = fs::read(&tmp).unwrap();
        assert_eq!(original.len(), saved.len(), "File sizes differ via markup");
        assert_eq!(original, saved, "File contents differ via markup");
        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn roundtrip_stage_parasol_via_markup() {
        let path = PathBuf::from("test_data/stage_parasol_00.msbt");
        let original = fs::read(&path).unwrap();
        let msbt = parse_msbt(&path).unwrap();

        let encoding = msbt.header.encoding;
        let num_strings = msbt.num_original_strings;

        let mut strings = vec![Vec::new(); num_strings];
        for entry in &msbt.entries {
            let tagged = markup::binary_to_tagged(&entry.raw_bytes);
            let binary = markup::tagged_to_binary(&tagged, encoding);
            assert_eq!(
                entry.raw_bytes, binary,
                "Markup roundtrip mismatch for label='{}' index={}",
                entry.label, entry.index
            );
            strings[entry.index] = binary;
        }

        let tmp = PathBuf::from("_test_sp_markup.msbt");
        write_msbt(&tmp, &msbt, &strings).unwrap();
        let saved = fs::read(&tmp).unwrap();
        assert_eq!(original.len(), saved.len(), "File sizes differ via markup");
        assert_eq!(original, saved, "File contents differ via markup");
        fs::remove_file(&tmp).ok();
    }

    /// Simulates the exact GUI save path:
    /// 1. parse_msbt (same as open_file)
    /// 2. binary_to_tagged for each entry (same as GuiEntry.tagged_text)
    /// 3. tagged_to_binary for each entry → all_strings[entry.index] (same as save_file)
    /// 4. write_msbt with all_strings (same as save_file after fix)
    /// 5. Byte-for-byte comparison with original
    #[test]
    fn gui_save_roundtrip_reading_name() {
        let path = PathBuf::from("test_data/reading_name.msbt");
        let original = fs::read(&path).unwrap();
        let msbt = parse_msbt(&path).unwrap();

        let encoding = msbt.header.encoding;
        let mut all_strings = vec![Vec::new(); msbt.num_original_strings];

        for entry in &msbt.entries {
            let tagged = markup::binary_to_tagged(&entry.raw_bytes);
            let bytes = markup::tagged_to_binary(&tagged, encoding);
            all_strings[entry.index] = bytes;
        }

        let tmp = PathBuf::from("_test_gui_rd.msbt");
        write_msbt(&tmp, &msbt, &all_strings).unwrap();
        let saved = fs::read(&tmp).unwrap();
        assert_eq!(original.len(), saved.len(), "File sizes differ");
        assert_eq!(original, saved, "File contents differ");
        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn gui_save_roundtrip_stage_parasol() {
        let path = PathBuf::from("test_data/stage_parasol_00.msbt");
        let original = fs::read(&path).unwrap();
        let msbt = parse_msbt(&path).unwrap();

        let encoding = msbt.header.encoding;
        let mut all_strings = vec![Vec::new(); msbt.num_original_strings];

        for entry in &msbt.entries {
            let tagged = markup::binary_to_tagged(&entry.raw_bytes);
            let bytes = markup::tagged_to_binary(&tagged, encoding);
            all_strings[entry.index] = bytes;
        }

        let tmp = PathBuf::from("_test_gui_sp.msbt");
        write_msbt(&tmp, &msbt, &all_strings).unwrap();
        let saved = fs::read(&tmp).unwrap();
        assert_eq!(original.len(), saved.len(), "File sizes differ");
        assert_eq!(original, saved, "File contents differ");
        fs::remove_file(&tmp).ok();
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub bom: [u8; 2],
    pub encoding: u8,       // 0 = UTF-8, 1 = UTF-16
    pub section_count: u16,
    pub file_size: u32,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub magic: [u8; 4],
    pub data: Vec<u8>,      // raw section data (without magic/size/padding header)
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub label: String,
    pub raw_bytes: Vec<u8>,
    pub index: usize,
    pub has_label: bool,
}

#[derive(Debug, Clone)]
pub struct MsbtFile {
    pub header: Header,
    pub sections: Vec<Section>,
    pub entries: Vec<Entry>,
    pub num_original_strings: usize,
    pub string_order: Vec<usize>, // maps entry index -> TXT2 string index
}

fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

fn write_u16_le(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_u32_le(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub fn parse_msbt(path: &Path) -> io::Result<MsbtFile> {
    let data = fs::read(path)?;
    if data.len() < 32 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File too small"));
    }

    let magic = &data[0..8];
    if &magic != b"MsgStdBn" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a valid MSBT file"));
    }

    let bom = [data[8], data[9]];
    if bom != [0xFF, 0xFE] {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Only little-endian MSBT supported"));
    }

    let encoding = data[12];
    let section_count = read_u16_le(&data, 14);
    let file_size = read_u32_le(&data, 18);

    let header = Header {
        bom,
        encoding,
        section_count,
        file_size,
    };

    let mut pos: usize = 32;
    let mut sections = Vec::new();
    let mut txt2_idx: Option<usize> = None;
    let mut lbl1_idx: Option<usize> = None;

    for _ in 0..section_count {
        if pos + 16 > data.len() {
            break;
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&data[pos..pos + 4]);
        let section_size = read_u32_le(&data, pos + 4) as usize;
        let data_start = pos + 16;
        let data_end = data_start + section_size;
        if data_end > data.len() {
            break;
        }
        let section_data = data[data_start..data_end].to_vec();
        sections.push(Section {
            magic,
            data: section_data,
        });

        if &magic == b"TXT2" {
            txt2_idx = Some(sections.len() - 1);
        }
        if &magic == b"LBL1" {
            lbl1_idx = Some(sections.len() - 1);
        }

        pos = data_end;
        let remainder = pos % 16;
        if remainder > 0 {
            pos += 16 - remainder;
        }
    }

    // Parse TXT2 strings
    let strings: Vec<Vec<u8>> = if let Some(idx) = txt2_idx {
        let sec = &sections[idx];
        let sec_data = &sec.data;
        if sec_data.len() < 4 {
            Vec::new()
        } else {
            let num_strings = read_u32_le(sec_data, 0) as usize;
            let offset_table_end = 4 + num_strings * 4;
            if offset_table_end > sec_data.len() {
                Vec::new()
            } else {
                let mut offsets = Vec::with_capacity(num_strings);
                for i in 0..num_strings {
                    offsets.push(read_u32_le(sec_data, 4 + i * 4) as usize);
                }
                let mut strings = Vec::with_capacity(num_strings);
                for i in 0..num_strings {
                    let start = offsets[i];
                    let end = if i + 1 < num_strings {
                        offsets[i + 1]
                    } else {
                        sec_data.len()
                    };
                    if start <= end && end <= sec_data.len() {
                        strings.push(sec_data[start..end].to_vec());
                    } else {
                        strings.push(Vec::new());
                    }
                }
                strings
            }
        }
    } else {
        Vec::new()
    };

    // Parse LBL1 labels
    struct LabelEntry {
        name: String,
        string_index: usize,
    }
    let labels: Vec<LabelEntry> = if let Some(idx) = lbl1_idx {
        let sec = &sections[idx];
        let sec_data = &sec.data;
        if sec_data.len() < 4 {
            Vec::new()
        } else {
            let num_groups = read_u32_le(sec_data, 0) as usize;
            let mut pos = 4;
            let mut group_offsets = Vec::new();
            for _ in 0..num_groups {
                if pos + 8 > sec_data.len() { break; }
                let num_labels = read_u32_le(sec_data, pos) as usize;
                let group_offset = read_u32_le(sec_data, pos + 4) as usize;
                group_offsets.push((num_labels, group_offset));
                pos += 8;
            }
            let mut labels = Vec::new();
            for (num_labels, group_offset) in group_offsets {
                let mut lp = group_offset;
                for _ in 0..num_labels {
                    if lp >= sec_data.len() { break; }
                    let name_len = sec_data[lp] as usize;
                    lp += 1;
                    if lp + name_len + 4 > sec_data.len() { break; }
                    let name_bytes = &sec_data[lp..lp + name_len];
                    let name = String::from_utf8_lossy(name_bytes).to_string();
                    lp += name_len;
                    let string_index = read_u32_le(sec_data, lp) as usize;
                    lp += 4;
                    labels.push(LabelEntry { name, string_index });
                }
            }
            labels
        }
    } else {
        Vec::new()
    };

    // Build entries from labels, preserving original order
    let mut used_indices = std::collections::HashSet::new();
    let mut entries = Vec::new();
    let mut string_order = Vec::new();

    for lbl in &labels {
        let raw = if lbl.string_index < strings.len() {
            strings[lbl.string_index].clone()
        } else {
            Vec::new()
        };
        used_indices.insert(lbl.string_index);
        string_order.push(lbl.string_index);
        entries.push(Entry {
            label: lbl.name.clone(),
            raw_bytes: raw,
            index: lbl.string_index,
            has_label: true,
        });
    }

    // Add unnamed strings not referenced by any label, preserving TXT2 order
    for i in 0..strings.len() {
        if !used_indices.contains(&i) {
            let raw = strings[i].clone();
            string_order.push(i);
            entries.push(Entry {
                label: format!("[{}]", i),
                raw_bytes: raw,
                index: i,
                has_label: false,
            });
        }
    }

    Ok(MsbtFile {
        header,
        sections,
        entries,
        num_original_strings: strings.len(),
        string_order,
    })
}

pub fn write_msbt(path: &Path, msbt: &MsbtFile, updated_strings: &[Vec<u8>]) -> io::Result<()> {
    let mut buf = Vec::new();

    // Header
    buf.extend_from_slice(b"MsgStdBn");
    buf.extend_from_slice(&msbt.header.bom);
    buf.extend_from_slice(&[0x00, 0x00]);
    buf.push(msbt.header.encoding);
    buf.push(0x03);
    write_u16_le(&mut buf, msbt.header.section_count as u16);
    write_u16_le(&mut buf, 0x0000);
    let file_size_pos = buf.len();
    write_u32_le(&mut buf, 0);
    buf.extend_from_slice(&[0u8; 10]);

    for section in &msbt.sections {
        buf.extend_from_slice(&section.magic);

        let section_data: Vec<u8> = if &section.magic == b"TXT2" {
            let mut sd = Vec::new();
            let count = updated_strings.len() as u32;
            write_u32_le(&mut sd, count);

            let offset_table_size = 4 + count as usize * 4;
            let mut running_total = offset_table_size;
            let offsets: Vec<u32> = updated_strings.iter().map(|s| {
                let off = running_total as u32;
                running_total += s.len();
                off
            }).collect();

            for &off in &offsets {
                write_u32_le(&mut sd, off);
            }
            for s in updated_strings {
                sd.extend_from_slice(s);
            }
            sd
        } else {
            section.data.clone()
        };

        let section_size = section_data.len() as u32;
        write_u32_le(&mut buf, section_size);
        buf.extend_from_slice(&[0u8; 8]);

        buf.extend_from_slice(&section_data);

        // Pad to 16-byte boundary with 0xAB
        let remainder = buf.len() % 16;
        if remainder > 0 {
            let pad_count = 16 - remainder;
            for _ in 0..pad_count {
                buf.push(0xAB);
            }
        }
    }

    // Update file size
    let file_size = buf.len() as u32;
    buf[file_size_pos..file_size_pos + 4].copy_from_slice(&file_size.to_le_bytes());

    fs::write(path, &buf)
}
