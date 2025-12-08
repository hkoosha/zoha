use gtk::CssProvider;
use gtk::prelude::CssProviderExt;
use log::trace;

pub(crate) fn set_css(screen: &gdk::Screen, css: &str) -> Result<(), glib::Error> {
    trace!("setting style");

    let provider = CssProvider::new();
    provider.load_from_data(css.as_ref())?;

    gtk::StyleContext::add_provider_for_screen(
        screen,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    return Ok(());
}
