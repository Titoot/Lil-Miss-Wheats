use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;

use crate::app::{AppState, Widgets};

pub fn show_find_dialog(
    widgets: &Rc<Widgets>,
    state: &Rc<RefCell<AppState>>,
    suppress: &Rc<Cell<bool>>,
) {
    let dialog = gtk4::Window::builder()
        .title("Find entries")
        .modal(true)
        .resizable(true)
        .default_width(400)
        .default_height(450)
        .transient_for(&widgets.window)
        .build();

    let search_entry = gtk4::SearchEntry::new();
    search_entry.set_placeholder_text(Some("Find what..."));
    search_entry.set_margin_start(8);
    search_entry.set_margin_end(8);
    search_entry.set_margin_top(8);

    let result_list = gtk4::ListBox::new();
    let scroll = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .margin_start(8)
        .margin_end(8)
        .build();
    scroll.set_child(Some(&result_list));

    let status_label = gtk4::Label::new(Some("Type to search"));
    status_label.set_xalign(0.0);
    status_label.set_margin_start(8);
    status_label.set_margin_end(8);
    status_label.set_margin_bottom(8);

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    vbox.append(&search_entry);
    vbox.append(&scroll);
    vbox.append(&status_label);

    dialog.set_child(Some(&vbox));

    let suppress_results: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let filtered_indices: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));

    // Search changed: filter entries and rebuild result list
    {
        let state = state.clone();
        let result_list = result_list.clone();
        let status_label = status_label.clone();
        let suppress_results = suppress_results.clone();
        let filtered_indices = filtered_indices.clone();

        search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            let s = state.borrow();

            suppress_results.set(true);
            while let Some(child) = result_list.first_child() {
                result_list.remove(&child);
            }
            suppress_results.set(false);

            if query.is_empty() {
                filtered_indices.borrow_mut().clear();
                status_label.set_text("Type to search");
                return;
            }

            let query_lower = query.to_lowercase();
            let mut indices = Vec::new();

            for (pos, gui_entry) in s.entries.iter().enumerate() {
                let matches_label = gui_entry.label.to_lowercase().contains(&query_lower);
                let matches_text = gui_entry.tagged_text.to_lowercase().contains(&query_lower);

                if !matches_label && !matches_text {
                    continue;
                }

                indices.push(pos);

                let context = matching_context(&gui_entry.tagged_text, &query);

                let label_w = gtk4::Label::new(None);
                label_w.set_markup(&format!(
                    "<b>{}</b>",
                    glib::markup_escape_text(&gui_entry.label)
                ));
                label_w.set_xalign(0.0);

                let context_w = gtk4::Label::new(Some(&context));
                context_w.set_xalign(0.0);
                context_w.set_ellipsize(gtk4::pango::EllipsizeMode::End);

                let row_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
                row_box.set_margin_start(6);
                row_box.set_margin_end(6);
                row_box.set_margin_top(4);
                row_box.set_margin_bottom(4);
                row_box.append(&label_w);
                row_box.append(&context_w);

                let row = gtk4::ListBoxRow::new();
                row.set_child(Some(&row_box));

                result_list.append(&row);
            }

            *filtered_indices.borrow_mut() = indices;
            let count = filtered_indices.borrow().len();
            status_label.set_text(&format!(
                "{} match{} found",
                count,
                if count == 1 { "" } else { "es" }
            ));
        });
    }

    // Row selected: select entry in main window and close
    {
        let state = state.clone();
        let widgets = widgets.clone();
        let suppress = suppress.clone();
        let dialog = dialog.clone();
        let suppress_results = suppress_results.clone();
        let filtered_indices = filtered_indices.clone();

        result_list.connect_row_selected(move |_list, row| {
            if suppress_results.get() {
                return;
            }
            let Some(row) = row else { return };

            let row_pos = row.index() as usize;
            let entry_idx = filtered_indices
                .borrow()
                .get(row_pos)
                .copied()
                .unwrap_or(0);

            {
                let mut s = state.borrow_mut();
                s.selected_index = entry_idx;
            }

            let s = state.borrow();
            crate::app::select_entry(&s, &widgets, &suppress);

            dialog.close();
        });
    }

    // Escape key closes the dialog
    {
        let dialog = dialog.clone();
        let key_ctrl = gtk4::EventControllerKey::new();
        key_ctrl.connect_key_pressed(move |_, keyval, _keycode, state_mod| {
            if state_mod == gtk4::gdk::ModifierType::empty()
                && keyval == gtk4::gdk::Key::Escape
            {
                dialog.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        search_entry.add_controller(key_ctrl);
    }

    dialog.present();
}

fn matching_context(text: &str, query: &str) -> String {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(byte_pos) = text_lower.find(&query_lower) {
        let char_pos = text[..byte_pos].chars().count();
        let total_chars = text.chars().count();
        let query_chars = query.chars().count();

        let context_before = 25;
        let context_after = 35;

        let start_char = char_pos.saturating_sub(context_before);
        let end_char = std::cmp::min(char_pos + query_chars + context_after, total_chars);

        let text_chars: Vec<char> = text.chars().collect();

        let mut result = String::new();
        if start_char > 0 {
            result.push('…');
        }
        for &c in &text_chars[start_char..char_pos] {
            result.push(c);
        }
        for &c in &text_chars[char_pos..char_pos + query_chars] {
            result.push(c);
        }
        for &c in &text_chars[char_pos + query_chars..end_char] {
            result.push(c);
        }
        if end_char < total_chars {
            result.push('…');
        }
        result
    } else {
        let total_chars = text.chars().count();
        let preview_chars = 60;
        if total_chars <= preview_chars {
            text.to_string()
        } else {
            text.chars().take(preview_chars).collect::<String>() + "…"
        }
    }
}
