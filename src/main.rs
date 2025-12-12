mod app;
mod click;
mod interval;
mod icon_names {
    pub use shipped::*;
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}

use app::AppModel;
use relm4::RelmApp;

fn main() {
    relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);

    let app = RelmApp::new("com.maskedd.click");
    app.run::<AppModel>(());
}
