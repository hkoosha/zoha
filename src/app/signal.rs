use crate::app::context;
use crate::err::ZohaError;
use dbus::Message;
use dbus::blocking::Connection;
use dbus::channel::Sender;
use gdk::gio::{
    DBusSignalFlags,
    SignalSubscription,
};
use gdk::prelude::ApplicationExt;
use gtk::prelude::{
    GtkWindowExt,
    WidgetExt,
};
use gtk::{
    Application,
    ApplicationWindow,
};
use log::{
    debug,
    error,
    info,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{
    Duration, 
    SystemTime,
};

#[derive(Debug)]
pub struct DBusDest<'a> {
    pub interface: &'a str,
    pub member: &'a str,
    pub path: &'a str,
}

pub fn connect_gdk_dbus(
    ctx: &Rc<RefCell<context::ZohaCtx>>,
    app: &Application,
    dest: &DBusDest,
) -> SignalSubscription {
    let ctx = Rc::clone(ctx);

    return app
        .dbus_connection()
        .expect("could not get a dbus connection")
        .subscribe_to_signal(
            None,
            Some(dest.interface),
            Some(dest.member),
            Some(dest.path),
            None, // arg0
            DBusSignalFlags::NONE,
            move |_| {
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
    }
    else {
        window.show_all();
        window.present();
        window.move_(ctx.x, ctx.y);
        ctx.showing = true;
    }

    ctx.last_toggle = SystemTime::now();
}

pub fn send_toggle_signal_through_dbus(
    dest: &DBusDest
) -> Result<(), ZohaError> {
    debug!("sending dbus signal");

    return match Connection::new_session()?.send(new_signal(dest)) {
        Ok(_) => {
            debug!("dbus signal sent");
            Ok(())
        }
        Err(_) => Err(ZohaError::UnknownDBus),
    };
}

pub(crate) fn new_signal(dest: &DBusDest) -> Message {
    let signal = Message::new_signal(dest.path, dest.interface, dest.member)
        .expect("failed to construct dbus signal");

    return signal;
}
