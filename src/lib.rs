#![allow(clippy::needless_return)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::rc::Rc;

use dbus::blocking::Connection;
use dbus::channel::Sender;
use dbus::Message;
use eyre::ContextCompat;
use eyre::eyre;
use gdk::Display;
use gdk::prelude::MonitorExt;
use gtk::AccelGroup;
use gtk::Application;
use gtk::ApplicationWindow;
use gtk::gio::DBusSignalFlags;
use gtk::Notebook;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ContainerExt;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::NotebookExt;
use gtk::prelude::WidgetExt;
use log::debug;

use crate::config::cfg::ZohaCfg;
use crate::ui::actions::set_app_actions;
use crate::ui::actions::set_win_actions;
use crate::ui::terminal::ZohaTerminal;
use crate::ui::window::add_tab;
use crate::ui::window::create_notebook;
use crate::ui::window::create_window;
use crate::ui::window::init_window;
use crate::ui::window::on_page_reorder;

pub mod config;
pub mod ui;
#[cfg(feature = "hack")]
pub mod hack;


pub const DBUS_INTERFACE: &str = "io.koosha.zoha";
pub const DBUS_MEMBER: &str = "zoha";
pub const DBUS_PATH: &str = "/io/koosha/zoha";

struct TabCounter(usize);

pub struct ZohaCtx {
    pub cfg: Rc<ZohaCfg>,
    pub font_scale: f64,
    pub fullscreen: bool,
    pub accel_group: AccelGroup,
    pub showing: bool,
    pub terminals: Rc<RefCell<HashMap<u32, ZohaTerminal>>>,
    pub transparency_enabled: bool,
    tab_counter: Rc<RefCell<TabCounter>>,
    window: Option<ApplicationWindow>,
    notebook: Option<Notebook>,
}

impl Debug for ZohaCtx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ZohaCtx[fullscreen={}, scaling_factor={}, accel_group={:?}, window={}]",
            self.fullscreen,
            self.font_scale,
            self.accel_group,
            if self.window.is_some() {
                "set"
            } else {
                "unset"
            }
        )
    }
}

impl ZohaCtx {
    pub fn new(cfg: Rc<ZohaCfg>) -> Self {
        let fullscreen = cfg.display.fullscreen;
        return Self {
            cfg,
            font_scale: 1.0,
            fullscreen,
            accel_group: AccelGroup::new(),
            showing: true,
            terminals: Rc::new(RefCell::new(HashMap::new())),
            transparency_enabled: true,
            window: None,
            notebook: None,
            tab_counter: Rc::new(RefCell::new(TabCounter(1))),
        };
    }

    pub fn set_window(&mut self,
                      window: ApplicationWindow) -> eyre::Result<()> {
        if self.window.is_some() {
            return Err(eyre!("window already set"));
        }

        debug!("setting window");
        window.add_accel_group(&self.accel_group);
        self.window = Some(window);

        return Ok(());
    }

    pub fn set_notebook(&mut self,
                        notebook: Notebook) {
        if self.notebook.is_some() {
            panic!("notebook already set")
        }

        debug!("setting notebook");
        self.notebook = Some(notebook);
    }

    pub fn get_window(&self) -> Option<&ApplicationWindow> {
        return self.window.as_ref();
    }

    pub fn get_notebook(&self) -> Option<&Notebook> {
        return self.notebook.as_ref();
    }

    pub fn font_inc(&mut self) {
        let old_size = self.font_scale;
        self.font_scale += 0.1;

        debug!("font scale inc {} => {}", old_size, self.font_scale);
    }

    pub fn font_dec(&mut self) {
        let old_size = self.font_scale;
        self.font_scale -= 0.1;

        debug!("font scale dec {} => {}", old_size, self.font_scale);
    }

    pub fn font_reset(&mut self) {
        let old_size = self.font_scale;
        self.font_scale = 1.0;

        debug!("font scale reset {} => {}", old_size, self.font_scale);
    }

    pub fn issue_tab_number(&self) -> usize {
        let num: usize = self.tab_counter.borrow().0;
        self.tab_counter.borrow_mut().0 += 1;
        return num;
    }
}


pub fn on_app_activate(ctx: &Rc<RefCell<ZohaCtx>>,
                       app: &Application) -> eyre::Result<()> {
    let window: ApplicationWindow = create_window(&ctx.borrow().cfg, app).build();

    // let ctx_on_focus = Rc::clone(ctx);
    // window.connect_activate_focus(move |_| {
    //     match ctx_on_focus.borrow().get_notebook() {
    //         None => eprintln!("missing notebook on window activate"),
    //         Some(notebook) => {
    //             let page = notebook.page();
    //
    //             if page < 0 {
    //                 eprintln!("no active page on notebook on window focus");
    //                 return;
    //             }
    //
    //             match ctx_on_focus.borrow().terminals.borrow().get(&(page as usize)) {
    //                 None => eprintln!("missing term on window focus: {}", page),
    //                 Some(term) => {
    //                     term.vte.grab_focus();
    //                 }
    //             }
    //         }
    //     };
    // });

    if let Err(err) = init_window(&mut ctx.borrow_mut(), window) {
        if format!("{}", err) == "window already set" {
            return Err(eyre!("app already active"));
        }
    }

    set_app_actions(&ctx.borrow(), app);
    set_win_actions(ctx);

    create_notebook(&mut ctx.borrow_mut());

    add_tab(ctx, !ctx.borrow().cfg.display.start_hidden);

    let reorder_ctx = Rc::clone(ctx);
    ctx.borrow().get_notebook().unwrap().connect_page_reordered(move |_, child, index| {
        on_page_reorder(&reorder_ctx, child, index);
    });

    ctx.borrow().get_window().unwrap().set_child(Some(ctx.borrow().get_notebook().unwrap()));

    if ctx.borrow().cfg.display.start_hidden {
        ctx.borrow().get_window().unwrap().hide();
        ctx.borrow_mut().showing = false;
    } else {
        ctx.borrow().get_window().unwrap().show_all();
    }

    return Ok(());
}

pub fn connect_gdk_dbus(ctx: &Rc<RefCell<ZohaCtx>>,
                        app: &Application) {
    let app: Application = app.clone();
    let ctx = Rc::clone(ctx);
    app.dbus_connection()
        .expect("could not get a dbus connection")
        .signal_subscribe(
            None,
            Some(DBUS_INTERFACE), // interface_name
            Some(DBUS_MEMBER), // member
            Some(DBUS_PATH),
            None, // arg0
            DBusSignalFlags::NONE,
            move |_, _, _, _, _, _| {
                toggle(&ctx);
            },
        );
}

pub fn toggle(ctx: &Rc<RefCell<ZohaCtx>>) {
    let mut ctx = ctx.borrow_mut();

    let window: &ApplicationWindow = ctx
        .get_window()
        .expect("application window missing while trying to toggle visibility");

    if ctx.showing {
        window.hide();
        ctx.showing = false;
    } else {
        window.show_all();
        window.present();
        ctx.showing = true;
    }
}

pub fn send_toggle_signal_through_dbus() -> eyre::Result<()> {
    let conn: Connection = Connection::new_session()?;
    return match conn.send(new_signal()) {
        Ok(_) => Ok(()),
        Err(_) => Err(eyre!("failed to send dbus signal")),
    };
}

pub(crate) fn new_signal() -> Message {
    let signal = Message::new_signal(
        DBUS_PATH,
        DBUS_INTERFACE,
        DBUS_MEMBER,
    ).expect("failed to construct dbus signal");

    return signal;
}

pub fn list_monitors() -> eyre::Result<Vec<String>> {
    let display: Display = Display::default().wrap_err_with(|| "could not get display")?;

    let mut monitors = vec![];
    for m in 0..display.n_monitors() {
        if let Some(monitor) = display.monitor(m) {
            let model: String = monitor
                .model()
                .map(|it| it.to_string())
                .unwrap_or_else(|| "?".to_string());

            monitors.push(format!("{} - {}", m, model));
        }
    }

    return Ok(monitors);
}

#[cfg(feature = "hack")]
pub fn list_keycodes() -> Vec<&'static str> {
    return vec![
        "Key0",
        "Key1",
        "Key2",
        "Key3",
        "Key4",
        "Key5",
        "Key6",
        "Key7",
        "Key8",
        "Key9",
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
        "G",
        "H",
        "I",
        "J",
        "K",
        "L",
        "M",
        "N",
        "O",
        "P",
        "Q",
        "R",
        "S",
        "T",
        "U",
        "V",
        "W",
        "X",
        "Y",
        "Z",
        "F1",
        "F2",
        "F3",
        "F4",
        "F5",
        "F6",
        "F7",
        "F8",
        "F9",
        "F10",
        "F11",
        "F12",
        "Escape",
        "Space",
        "LControl",
        "RControl",
        "LShift",
        "RShift",
        "LAlt",
        "RAlt",
        "Meta",
        "Enter",
        "Up",
        "Down",
        "Left",
        "Right",
        "Backspace",
        "CapsLock",
        "Tab",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        "Insert",
        "Delete",
        "Numpad0",
        "Numpad1",
        "Numpad2",
        "Numpad3",
        "Numpad4",
        "Numpad5",
        "Numpad6",
        "Numpad7",
        "Numpad8",
        "Numpad9",
        "NumpadSubtract",
        "NumpadAdd",
        "NumpadDivide",
        "NumpadMultiply",
        "Grave",
        "Minus",
        "Equal",
        "LeftBracket",
        "RightBracket",
        "BackSlash",
        "Semicolon",
        "Apostrophe",
        "Comma",
        "Dot",
        "Slash",
    ];
}
