#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod msbt;
mod markup;
mod arabic;
mod gui_types;
mod gui_entry_list;
mod gui_hexview;

use gtk4::prelude::ApplicationExt;
use gtk4::prelude::ApplicationExtManual;

fn main() {
    let app = gtk4::Application::builder()
        .application_id("com.titoot.lilmisswheats")
        .build();

    app.connect_activate(app::build_ui);
    app.run();
}
