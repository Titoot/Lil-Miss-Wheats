use gtk4::prelude::*;
use crate::gui_types::GuiEntry;

pub fn rebuild(list: &gtk4::ListBox, entries: &[GuiEntry], selected: usize) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    for (_i, entry) in entries.iter().enumerate() {
        let label = gtk4::Label::new(Some(&entry.label));
        label.set_xalign(0.0);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        label.set_margin_start(6);
        label.set_margin_end(6);
        label.set_margin_top(4);
        label.set_margin_bottom(4);

        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&label));

        list.append(&row);
    }

    if let Some(row) = list.row_at_index(selected as i32) {
        list.select_row(Some(&row));
    }
}
