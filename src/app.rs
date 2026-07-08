use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::prelude::*;

use crate::msbt;
use crate::gui_entry_list;
use crate::gui_types::GuiEntry;

pub struct AppState {
    pub file_path: Option<PathBuf>,
    pub msbt: Option<msbt::MsbtFile>,
    pub entries: Vec<GuiEntry>,
    pub selected_index: usize,
    pub encoding: u8,
    pub reverse: bool,
    pub dirty: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            file_path: None,
            msbt: None,
            entries: Vec::new(),
            selected_index: 0,
            encoding: 0,
            reverse: false,
            dirty: false,
        }
    }
}

pub struct Widgets {
    pub window: gtk4::ApplicationWindow,
    pub entry_list: gtk4::ListBox,
    pub editor_label: gtk4::Label,
    pub editor_buffer: gtk4::TextBuffer,
    pub original_buffer: gtk4::TextBuffer,
    pub hex_buffer: gtk4::TextBuffer,
    pub status_label: gtk4::Label,
    pub file_label: gtk4::Label,
    pub reverse_check: gtk4::CheckButton,
    pub preview_label: gtk4::Label,
}

fn open_file(path: &Path, state: &mut AppState, widgets: &Widgets, suppress: &Cell<bool>) {
    match msbt::parse_msbt(path) {
        Ok(msbt_file) => {
            state.file_path = Some(path.to_path_buf());
            state.encoding = msbt_file.header.encoding;
            state.dirty = false;
            state.entries = msbt_file.entries.iter().map(|e| {
                let tagged = crate::markup::binary_to_tagged(&e.raw_bytes);
                GuiEntry {
                    label: e.label.clone(),
                    tagged_text: tagged.clone(),
                    original_tagged_text: tagged,
                    index: e.index,
                }
            }).collect();
            state.selected_index = 0;
            state.msbt = Some(msbt_file);

            let fname = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            widgets.file_label.set_text(&fname);
            widgets.status_label.set_text(&format!("Loaded {} entries", state.entries.len()));

            gui_entry_list::rebuild(&widgets.entry_list, &state.entries, state.selected_index);
            select_entry(state, widgets, suppress);
        }
        Err(e) => {
            widgets.status_label.set_text(&format!("Error: {}", e));
        }
    }
}

fn select_entry(state: &AppState, widgets: &Widgets, suppress: &Cell<bool>) {
    suppress.set(true);
    if let Some(entry) = state.entries.get(state.selected_index) {
        widgets.editor_label.set_text(&format!("Label: {}  (index {})", entry.label, entry.index));
        widgets.editor_buffer.set_text(&entry.tagged_text);
        widgets.original_buffer.set_text(&entry.original_tagged_text);
        widgets.reverse_check.set_active(state.reverse);
        let display = if state.reverse { apply_arabic_to_tagged(&entry.tagged_text) } else { entry.tagged_text.clone() };
        let bytes = crate::markup::tagged_to_binary(&display, state.encoding);
        crate::gui_hexview::fill_hex_buffer(&widgets.hex_buffer, &bytes);
    } else {
        widgets.editor_label.set_text("No entry selected");
        widgets.editor_buffer.set_text("");
        widgets.original_buffer.set_text("");
        widgets.reverse_check.set_active(false);
        widgets.hex_buffer.set_text("");
    }
    suppress.set(false);
    update_preview(state, widgets);
}

fn save_file(path: &Path, state: &mut AppState, widgets: &Widgets) {
    let Some(ref msbt) = state.msbt else {
        widgets.status_label.set_text("No file to save.");
        return;
    };

    let encoding = state.encoding;
    let mut all_strings: Vec<Vec<u8>> = vec![Vec::new(); msbt.num_original_strings];

    for entry in &state.entries {
        let idx = entry.index;
        if idx >= all_strings.len() {
            all_strings.resize(idx + 1, Vec::new());
        }
        let mut text = entry.tagged_text.clone();
        if state.reverse {
            text = apply_arabic_to_tagged(&text);
        }
        let bytes = crate::markup::tagged_to_binary(&text, encoding);
        all_strings[idx] = bytes;
    }

    match msbt::write_msbt(path, msbt, &all_strings) {
        Ok(()) => {
            state.dirty = false;
            let name = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            widgets.status_label.set_text(&format!("Saved to {}", name));
            widgets.file_label.set_text(&name);
            state.file_path = Some(path.to_path_buf());
        }
        Err(e) => {
            widgets.status_label.set_text(&format!("Save error: {}", e));
        }
    }
}

fn update_preview(state: &AppState, widgets: &Widgets) {
    let text = widgets.editor_buffer.text(
        &widgets.editor_buffer.start_iter(),
        &widgets.editor_buffer.end_iter(),
        false,
    ).to_string();

    if let Some(_entry) = state.entries.get(state.selected_index) {
        if state.reverse && !text.is_empty() {
            let preview = apply_arabic_to_tagged(&text);
            widgets.preview_label.set_markup(&format!(
                "<span foreground='#6699ff'>In-game preview (shaped + reversed):\n{}</span>",
                glib::markup_escape_text(&preview)
            ));
            widgets.preview_label.set_visible(true);
        } else {
            widgets.preview_label.set_visible(false);
        }
    } else {
        widgets.preview_label.set_visible(false);
    }
}

pub fn build_ui(app: &gtk4::Application) {
    let state = Rc::new(RefCell::new(AppState::default()));
    let suppress_change: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    // --- Window ---
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Lil' Miss Wheats")
        .default_width(1200)
        .default_height(700)
        .build();

    // --- Toolbar ---
    let open_btn = gtk4::Button::with_label("Open");
    let save_btn = gtk4::Button::with_label("Save");
    let save_as_btn = gtk4::Button::with_label("Save As...");
    let file_label = gtk4::Label::new(None);
    file_label.set_xalign(0.0);
    file_label.set_margin_end(12);
    let status_label = gtk4::Label::new(Some("Open an MSBT file to begin."));
    status_label.set_xalign(0.0);
    status_label.set_hexpand(true);

    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    status_box.append(&file_label);
    status_box.append(&status_label);

    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(8);
    toolbar.append(&open_btn);
    toolbar.append(&save_btn);
    toolbar.append(&save_as_btn);
    toolbar.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
    toolbar.append(&status_box);

    // --- Entry list ---
    let entry_list = gtk4::ListBox::new();
    entry_list.set_width_request(180);
    let scroll_list = gtk4::ScrolledWindow::builder()
        .margin_start(4)
        .margin_top(4)
        .margin_bottom(4)
        .min_content_height(100)
        .build();
    scroll_list.set_child(Some(&entry_list));

    // --- Editor ---
    let editor_label = gtk4::Label::new(None);
    editor_label.set_xalign(0.0);
    editor_label.set_margin_bottom(4);

    let editor_buffer = gtk4::TextBuffer::new(None);
    let editor_view = gtk4::TextView::with_buffer(&editor_buffer);
    editor_view.set_wrap_mode(gtk4::WrapMode::Char);
    editor_view.set_monospace(true);
    editor_view.set_direction(gtk4::TextDirection::Rtl);

    let scroll_editor = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .min_content_height(100)
        .build();
    scroll_editor.set_child(Some(&editor_view));

    let reverse_check = gtk4::CheckButton::with_label("Reverse display text on save");
    reverse_check.set_margin_top(4);

    let preview_label = gtk4::Label::new(None);
    preview_label.set_xalign(0.0);
    preview_label.set_wrap(true);
    preview_label.set_selectable(true);
    preview_label.set_margin_top(4);

    let editor_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    editor_box.set_margin_start(8);
    editor_box.set_margin_end(8);
    editor_box.set_margin_top(8);
    editor_box.set_margin_bottom(4);
    editor_box.append(&editor_label);
    editor_box.append(&scroll_editor);
    editor_box.append(&reverse_check);
    editor_box.append(&preview_label);

    // Tags info at the bottom
    let tags_info = gtk4::Label::new(Some(
        "Tags:  [TTS(\"tts\")]text - TTS voice text + display text   |   [SEP] - Textbox separator   |   [N] - Null byte (line break)   |   [A] [A_ALT] [B] [B_ALT] [Y] [+] [MINUS] [UP] [DOWN] [LEFT] [RIGHT] [SL] [SR]"
    ));
    tags_info.set_xalign(0.0);
    tags_info.set_wrap(true);
    tags_info.set_margin_start(8);
    tags_info.set_margin_end(8);
    tags_info.set_margin_bottom(8);

    // --- Original text viewer ---
    let original_buffer = gtk4::TextBuffer::new(None);
    let original_view = gtk4::TextView::with_buffer(&original_buffer);
    original_view.set_editable(false);
    original_view.set_cursor_visible(false);
    original_view.set_wrap_mode(gtk4::WrapMode::Char);
    original_view.set_monospace(true);

    let scroll_original = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(false)
        .min_content_height(100)
        .build();
    scroll_original.set_child(Some(&original_view));

    let original_frame = gtk4::Frame::builder()
        .label("Original text (uneditable)")
        .margin_start(8)
        .margin_end(8)
        .margin_bottom(8)
        .build();
    original_frame.set_child(Some(&scroll_original));

    let editor_panel = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    editor_panel.append(&editor_box);
    editor_panel.append(&tags_info);
    editor_panel.append(&original_frame);

    // --- Hex viewer ---
    let hex_buffer = gtk4::TextBuffer::new(None);
    let hex_view = gtk4::TextView::with_buffer(&hex_buffer);
    hex_view.set_monospace(true);
    hex_view.set_editable(false);
    hex_view.set_cursor_visible(false);
    hex_view.set_wrap_mode(gtk4::WrapMode::Char);

    let scroll_hex = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_width(320)
        .margin_start(4)
        .margin_end(4)
        .margin_top(4)
        .margin_bottom(4)
        .build();
    scroll_hex.set_child(Some(&hex_view));

    let hex_frame = gtk4::Frame::builder()
        .label("Hex (current entry bytes)")
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    hex_frame.set_child(Some(&scroll_hex));

    // --- Layout with paned splits ---
    let right_split = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    right_split.set_start_child(Some(&editor_panel));
    right_split.set_end_child(Some(&hex_frame));
    right_split.set_position(700);
    right_split.set_wide_handle(true);

    let main_split = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    main_split.set_start_child(Some(&scroll_list));
    main_split.set_end_child(Some(&right_split));
    main_split.set_position(220);
    main_split.set_wide_handle(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.append(&toolbar);
    content.append(&main_split);

    window.set_child(Some(&content));
    window.set_default_size(1200, 700);
    window.present();

    // --- Dark mode ---
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }
    let css = gtk4::CssProvider::new();
    css.load_from_data(
        "textview { background-color: #1e1e1e; color: #cccccc; }
         label { color: #cccccc; }
         checkbutton label { color: #cccccc; }
         listbox { background-color: #252526; }
         listbox row { color: #cccccc; }
         listbox row:selected { background-color: #094771; color: #ffffff; }
         entry { background-color: #3c3c3c; color: #cccccc; }
         button { background-color: #3c3c3c; color: #cccccc; }
         button:hover { background-color: #4a4a4a; }
         scrolledwindow { background-color: #1e1e1e; }
         scrollbar { background-color: #2d2d2d; }
         scrollbar slider { background-color: #424242; }
         scrollbar slider:hover { background-color: #555555; }
         frame { color: #999999; }
         frame border { border-color: #3c3c3c; }
         textview text { background-color: #1e1e1e; color: #cccccc; }
         separator { background-color: #3c3c3c; }
         paned separator { background-color: #3c3c3c; }
         paned separator:hover { background-color: #555555; }"
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::prelude::WidgetExt::display(&window),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // --- Widget registry ---
    let widgets = Rc::new(Widgets {
        window: window.clone(),
        entry_list: entry_list.clone(),
        editor_label: editor_label.clone(),
        editor_buffer: editor_buffer.clone(),
        original_buffer: original_buffer.clone(),
        hex_buffer: hex_buffer.clone(),
        status_label: status_label.clone(),
        file_label: file_label.clone(),
        reverse_check: reverse_check.clone(),
        preview_label: preview_label.clone(),
    });

    // --- Drag-and-drop: open .msbt files ---
    {
        let state = state.clone();
        let widgets = widgets.clone();
        let sup = suppress_change.clone();
        let drop_target = gtk4::DropTarget::new(
            gtk4::gio::File::static_type(),
            gtk4::gdk::DragAction::COPY,
        );
        drop_target.connect_drop(move |_, value, _x, _y| {
            if let Ok(file) = value.get::<gtk4::gio::File>() {
                if let Some(path) = file.path() {
                    if path.extension().map_or(false, |ext| ext == "msbt") {
                        let mut s = state.borrow_mut();
                        open_file(&path, &mut s, &widgets, &sup);
                        return true;
                    }
                }
            }
            false
        });
        window.add_controller(drop_target);
    }

    // --- Close-request: unsaved changes prompt ---
    {
        let state = state.clone();
        window.connect_close_request(move |win| {
            if !state.borrow().dirty {
                return glib::Propagation::Proceed;
            }
            let alert = gtk4::AlertDialog::builder()
                .message("Unsaved Changes")
                .detail("You have unsaved changes. Discard them and close?")
                .modal(true)
                .buttons(["Discard changes", "Cancel"])
                .build();
            let win2 = win.clone();
            let state2 = state.clone();
            alert.choose(Some(win), None::<&gtk4::gio::Cancellable>, move |result| {
                if let Ok(0) = result {
                    state2.borrow_mut().dirty = false;
                    win2.close();
                }
            });
            glib::Propagation::Stop
        });
    }

    // --- Signal: Open ---
    {
        let state = state.clone();
        let widgets = widgets.clone();
        let sup = suppress_change.clone();
        open_btn.connect_clicked(move |_| {
            let s2 = state.clone();
            let w2 = widgets.clone();
            let sup2 = sup.clone();
            let filter = {
                let f = gtk4::FileFilter::new();
                f.set_name(Some("MSBT files (*.msbt)"));
                f.add_pattern("*.msbt");
                f
            };
            let dialog = gtk4::FileDialog::builder()
                .title("Open MSBT file")
                .default_filter(&filter)
                .build();
            dialog.open(Some(&widgets.window), None::<&gtk4::gio::Cancellable>, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let mut s = s2.borrow_mut();
                        open_file(&path, &mut s, &w2, &sup2);
                    }
                }
            });
        });
    }

    // --- Signal: Save ---
    {
        let state = state.clone();
        let widgets = widgets.clone();
        save_btn.connect_clicked(move |_| {
            let s = state.borrow();
            if let Some(ref path) = s.file_path.clone() {
                drop(s);
                let mut s2 = state.borrow_mut();
                save_file(&path, &mut s2, &widgets);
            } else {
                widgets.status_label.set_text("No file open. Use Save As...");
            }
        });
    }

    // --- Signal: Save As... ---
    {
        let state = state.clone();
        let widgets = widgets.clone();
        let sup = suppress_change.clone();
        save_as_btn.connect_clicked(move |_| {
            let s2 = state.clone();
            let w2 = widgets.clone();
            let sup2 = sup.clone();
            let filter = {
                let f = gtk4::FileFilter::new();
                f.set_name(Some("MSBT files (*.msbt)"));
                f.add_pattern("*.msbt");
                f
            };
            let dialog = gtk4::FileDialog::builder()
                .title("Save MSBT file")
                .default_filter(&filter)
                .build();
            dialog.save(Some(&widgets.window), None::<&gtk4::gio::Cancellable>, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let mut s = s2.borrow_mut();
                        save_file(&path, &mut s, &w2);
                        if let Some(ref p) = s.file_path.clone() {
                            open_file(p, &mut s, &w2, &sup2);
                        }
                    }
                }
            });
        });
    }

    // --- Signal: Reverse checkbox ---
    {
        let state = state.clone();
        let widgets = widgets.clone();
        let sup = suppress_change.clone();
        reverse_check.connect_toggled(move |btn| {
            if sup.get() { return; }
            let mut s = state.borrow_mut();
            s.reverse = btn.is_active();
            s.dirty = true;
            let encoding = s.encoding;
            let text = widgets.editor_buffer.text(
                &widgets.editor_buffer.start_iter(),
                &widgets.editor_buffer.end_iter(),
                false,
            ).to_string();
            let display_text = if btn.is_active() { apply_arabic_to_tagged(&text) } else { text };
            let bytes = crate::markup::tagged_to_binary(&display_text, encoding);
            crate::gui_hexview::fill_hex_buffer(&widgets.hex_buffer, &bytes);
            drop(s);
            let s2 = state.borrow();
            update_preview(&s2, &widgets);
        });
    }

    // --- Signal: Editor text changed ---
    {
        let state = state.clone();
        let widgets = widgets.clone();
        let sup = suppress_change.clone();
        editor_buffer.connect_changed(move |buf| {
            if sup.get() {
                return;
            }
            let mut s = match state.try_borrow_mut() {
                Ok(s) => s,
                Err(_) => return,
            };
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false)
                .to_string();
            let idx = s.selected_index;
            let reverse = s.reverse;
            if let Some(entry) = s.entries.get_mut(idx) {
                entry.tagged_text = text.clone();
                s.dirty = true;
            }
            let display_text = if reverse { apply_arabic_to_tagged(&text) } else { text };
            let bytes = crate::markup::tagged_to_binary(&display_text, s.encoding);
            crate::gui_hexview::fill_hex_buffer(&widgets.hex_buffer, &bytes);
            drop(s);
            let s2 = state.borrow();
            update_preview(&s2, &widgets);
        });
    }

    // --- Signal: Entry list selection ---
    {
        let state = state.clone();
        let widgets = widgets.clone();
        let sup = suppress_change.clone();
        entry_list.connect_row_activated(move |_list, row| {
            let idx = row.index() as usize;
            let mut s = state.borrow_mut();
            if idx < s.entries.len() {
                s.selected_index = idx;
            }
            drop(s);
            let s2 = state.borrow();
            select_entry(&s2, &widgets, &sup);
        });
    }
}

pub(crate) fn apply_arabic_to_tagged(tagged: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = tagged.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Collect a run of plain (non-tag) text
        let start = i;
        while i < chars.len() && chars[i] != '[' { i += 1; }
        if i > start {
            let plain: String = chars[start..i].iter().collect();
            result.push_str(&crate::arabic::process_arabic(&plain));
        }
        if i >= chars.len() { break; }

        // Handle tag
        if i + 5 < chars.len()
            && chars[i] == '['
            && chars[i + 1] == 'T'
            && chars[i + 2] == 'T'
            && chars[i + 3] == 'S'
            && chars[i + 4] == '('
            && chars[i + 5] == '"'
        {
            let mut j = i;
            while j < chars.len() && chars[j] != ']' { j += 1; }
            if j < chars.len() { j += 1; }

            let tts_tag: String = chars[i..j].iter().collect();
            result.push_str(&tts_tag);

            let display_start = j;
            while j < chars.len() {
                if (j + 4 < chars.len() && chars[j] == '[' && chars[j + 1] == 'S' && chars[j + 2] == 'E' && chars[j + 3] == 'P')
                    || (j + 5 < chars.len() && chars[j] == '[' && chars[j + 1] == 'T' && chars[j + 2] == 'T' && chars[j + 3] == 'S')
                    || (j + 2 < chars.len() && chars[j] == '[' && chars[j + 1] == '-' && chars[j + 2] == ']')
                {
                    break;
                }
                j += 1;
            }

            let display_text: String = chars[display_start..j].iter().collect();
            let reversed = crate::arabic::process_arabic(&display_text);
            result.push_str(&reversed);
            i = j;
        } else {
            // Other tag ([SEP], [N], [-], etc.) — keep as-is
            let tag_start = i;
            while i < chars.len() && chars[i] != ']' { i += 1; }
            if i < chars.len() { i += 1; }
            let tag: String = chars[tag_start..i].iter().collect();
            result.push_str(&tag);
        }
    }

    result
}
