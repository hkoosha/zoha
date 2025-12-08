use crate::app::context;
use dbus::Message;
use dbus::blocking::Connection;
use dbus::channel::Sender;
use eyre::eyre;
use gdk::gio::DBusSignalFlags;
use gdk::prelude::ApplicationExt;
use gtk::prelude::{GtkWindowExt, WidgetExt};
use gtk::{Application, ApplicationWindow};
use log::{debug, error, info};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

pub const DBUS_INTERFACE: &str = "io.koosha.zoha";
pub const DBUS_MEMBER: &str = "zoha";
pub const DBUS_PATH: &str = "/io/koosha/zoha";

pub fn connect_gdk_dbus(ctx: &Rc<RefCell<context::ZohaCtx>>, app: &Application) {
    let ctx = Rc::clone(ctx);

    app.dbus_connection()
        .expect("could not get a dbus connection")
        .signal_subscribe(
            None,
            Some(DBUS_INTERFACE), // interface_name
            Some(DBUS_MEMBER),    // member
            Some(DBUS_PATH),
            None, // arg0
            DBusSignalFlags::NONE,
            move |_, _, _, _, _, _| {
                toggle(&ctx);
            },
        );
}

pub(crate) fn toggle(ctx: &Rc<RefCell<context::ZohaCtx>>) {
    let mut ctx = ctx.borrow_mut();

    let wait = 50u128;
    let now = SystemTime::now();
    let diff = now
        .duration_since(ctx.last_toggle)
        .unwrap_or_else(|e| {
            error!("failed to get system time: {}", e);
            Duration::from_secs(0)
        })
        .as_millis();

    if diff < wait {
        info!(
            "not toggling, as toggle event already happened less than milliseconds before: {}",
            wait
        );

        return;
    }

    let window: &ApplicationWindow = ctx
        .get_window()
        .expect("application window missing while trying to toggle visibility");

    debug!(
        "will toggle: at={}:{}, will_show={}",
        ctx.x, ctx.y, !ctx.showing
    );

    if ctx.showing {
        window.hide();
        ctx.showing = false;
    } else {
        window.show_all();
        window.present();
        window.move_(ctx.x, ctx.y);
        ctx.showing = true;
    }

    ctx.last_toggle = SystemTime::now();
}

pub fn send_toggle_signal_through_dbus() -> eyre::Result<()> {
    debug!("sending dbus signal");

    return match Connection::new_session()?.send(new_signal()) {
        Ok(_) => {
            debug!("dbus signal sent");
            Ok(())
        }
        Err(_) => Err(eyre!("failed to send dbus signal")),
    };
}

pub(crate) fn new_signal() -> Message {
    let signal = Message::new_signal(DBUS_PATH, DBUS_INTERFACE, DBUS_MEMBER)
        .expect("failed to construct dbus signal");

    return signal;
}
