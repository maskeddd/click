fn main() {
    relm4_icons_build::bundle_icons(
        "icon_names.rs",
        Some("com.maskedd.click"),
        None::<&str>,
        None::<&str>,
        ["mouse-click", "menu"],
    );
}
