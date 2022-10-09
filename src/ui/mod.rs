use gtk::prelude::*;
use std::error::Error;

mod about_dialog;
mod bottom_bar;
mod browser_panel;
mod main_window_menu_bar;
mod top_bar;

const APP_ID: &str = "app.meadowlark.Meadowlark";

//const MEADOWLARK_ICONS_FONT: &[u8] = include_bytes!("resources/fonts/Meadowlark-Icons.ttf");
//const MIN_SANS_MEDIUM: &[u8] = include_bytes!("resources/fonts/MinSans-Medium.otf");

pub fn run_ui() -> Result<(), Box<dyn Error>> {
    gtk::gio::resources_register_include!("compiled.gresource").unwrap();

    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .resource_base_path("/app/meadowlark/Meadowlark")
        .build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();

    Ok(())
}

fn setup_style() {
    let default_display = gtk::gdk::Display::default().expect("Could not connect to a display.");

    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/app/meadowlark/Meadowlark/default-dark.css");
    gtk::StyleContext::add_provider_for_display(
        &default_display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    #[cfg(not(target_os = "macos"))]
    {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/app/meadowlark/Meadowlark/font-sizes.css");
        gtk::StyleContext::add_provider_for_display(
            &default_display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // For some reason, fonts and icons appear smaller on MacOS than they should, so use
    // a CSS file with bigger sizes.
    #[cfg(target_os = "macos")]
    {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/app/meadowlark/Meadowlark/font-sizes-macos.css");
        gtk::StyleContext::add_provider_for_display(
            &default_display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let icon_theme = gtk::IconTheme::for_display(&default_display);
    icon_theme.add_resource_path("/app/meadowlark/Meadowlark/icons");
}

fn build_ui(app: &gtk::Application) {
    setup_style();

    let main_window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Meadowlark")
        .width_request(1280)
        .height_request(800)
        .show_menubar(false)
        .icon_name("meadowlark")
        .build();

    let main_box = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();

    let top_bar = top_bar::setup();
    main_box.append(&top_bar);

    let center_contents =
        gtk::Box::builder().orientation(gtk::Orientation::Horizontal).vexpand(true).build();
    center_contents.append(&browser_panel::browser_panel_tabs::setup());
    main_box.append(&center_contents);

    let bottom_bar = bottom_bar::setup();
    main_box.append(&bottom_bar);

    main_window.set_child(Some(&main_box));

    about_dialog::setup(&main_window);

    // Present the window
    main_window.present();
}
