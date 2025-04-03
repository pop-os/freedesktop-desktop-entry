use freedesktop_desktop_entry as fde;

fn main() {
    let locales = fde::get_languages_from_env();
    let desktop_entries = fde::desktop_entries(&locales);

    for arg in std::env::args().skip(1) {
        let arg = fde::unicase::Ascii::new(arg.as_str());

        let desktop_entry =
            fde::find_app_by_id(&desktop_entries, arg).expect("could not find appid");

        let icon_source = fde::IconSource::from_unknown(desktop_entry.icon().unwrap_or_default());

        println!("{arg}: {desktop_entry:#?} with icon {icon_source:?}");
    }
}
