#![allow(clippy::needless_return)]
#[cfg(all(feature = "hack", not(target_os = "linux")))]
compile_error!("feature 'hack' can oly be enabled on linux");

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::Arc;
use cacao::foundation::id;
use cacao::objc::{class, msg_send, sel, sel_impl};

use eyre::ContextCompat;
use eyre::eyre;
use gdk::Display;
use gdk::prelude::MonitorExt;
use gtk::AccelGroup;
use gtk::Application;
use gtk::ApplicationWindow;
#[cfg(target_os = "linux")]
use gtk::gio::DBusSignalFlags;
use gtk::Notebook;
#[cfg(target_os = "linux")]
use gtk::prelude::ApplicationExt;
use gtk::prelude::ContainerExt;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::NotebookExt;
use gtk::prelude::WidgetExt;
use log::{debug, trace};

#[cfg(target_os = "linux")]
use dbus::blocking::Connection;
#[cfg(target_os = "linux")]
use dbus::channel::Sender;
#[cfg(target_os = "linux")]
use dbus::Message;

use crate::config::cfg::ZohaCfg;
use crate::config::color::Pallet;
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


#[cfg(target_os = "linux")]
pub const DBUS_INTERFACE: &str = "io.koosha.zoha";
#[cfg(target_os = "linux")]
pub const DBUS_MEMBER: &str = "zoha";
#[cfg(target_os = "linux")]
pub const DBUS_PATH: &str = "/io/koosha/zoha";

struct TabCounter(usize);

pub struct ZohaCtx {
    pub cfg: Arc<ZohaCfg>,
    pub font_scale: f64,
    pub fullscreen: bool,
    pub accel_group: AccelGroup,
    pub showing: bool,
    pub terminals: Arc<RefCell<HashMap<u32, ZohaTerminal>>>,
    pub transparency_enabled: bool,
    tab_counter: Arc<RefCell<TabCounter>>,
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
    pub fn new(cfg: Arc<ZohaCfg>) -> Self {
        let fullscreen = cfg.display.fullscreen;
        return Self {
            cfg,
            font_scale: 1.0,
            fullscreen,
            accel_group: AccelGroup::new(),
            showing: true,
            terminals: Arc::new(RefCell::new(HashMap::new())),
            transparency_enabled: true,
            window: None,
            notebook: None,
            tab_counter: Arc::new(RefCell::new(TabCounter(1))),
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


pub fn on_app_activate(ctx: &Arc<RefCell<ZohaCtx>>,
                       app: &Application) -> eyre::Result<()> {
    let window: ApplicationWindow = create_window(&ctx.borrow().cfg, app).build();

    // let ctx_on_focus = Arc::clone(ctx);
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

    let reorder_ctx = Arc::clone(ctx);
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

#[cfg(target_os = "linux")]
pub fn connect_gdk_dbus(ctx: &Arc<RefCell<ZohaCtx>>,
                        app: &Application) {
    let app: Application = app.clone();
    let ctx = Arc::clone(ctx);
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

pub fn toggle(ctx: &Arc<RefCell<ZohaCtx>>) {
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

    unsafe { macos_screens(); }
}

#[cfg(target_os = "linux")]
pub fn send_toggle_signal_through_dbus() -> eyre::Result<()> {
    debug!("sending dbus signal");
    let conn: Connection = Connection::new_session()?;
    return match conn.send(new_signal()) {
        Ok(_) => {
            trace!("dbus signal sent");
            Ok(())
        }
        Err(_) => Err(eyre!("failed to send dbus signal")),
    };
    return Ok(());
}

#[cfg(target_os = "linux")]
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

#[cfg(all(feature = "hack"))]
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

pub fn print_config(cfg: ZohaCfg) {

    let or_string = || "".to_string();

    println!("font.font = {}", cfg.font.font);

    // =================

    println!();

    println!("display.monitor = {}", cfg.display.monitor.unwrap_or_else(or_string));
    println!("display.title = {}", cfg.display.title);
    println!("display.x_pos = {}", cfg.display.x_pos);
    println!("display.y_pos = {}", cfg.display.y_pos);
    println!("display.width = {}",
             cfg.display.width.map(|it| it.to_string()).unwrap_or_else(or_string));
    println!("display.height = {}",
             cfg.display.height.map(|it| it.to_string()).unwrap_or_else(or_string));
    println!(
        "display.width_percentage = {}",
        cfg.display.width_percentage.map(|it| it.to_string()).unwrap_or_else(or_string),
    );
    println!(
        "display.height_percentage = {}",
        cfg.display.height_percentage.map(|it| it.to_string()).unwrap_or_else(or_string),
    );
    println!("display.start_hidden = {}", cfg.display.start_hidden);
    println!("display.skip_task_bar = {}", cfg.display.skip_task_bar);
    println!("display.always_on_top = {}", cfg.display.always_on_top);
    println!("display.sticky = {}", cfg.display.sticky);
    println!("display.fullscreen = {}", cfg.display.fullscreen);
    println!("display.tab_mode = {}", cfg.display.tab_mode);
    println!("display.tab_position = {}", cfg.display.tab_position);
    println!("display.tab_expand = {}", cfg.display.tab_expand);
    println!(
        "display.tab_title_num_characters = {}",
        cfg.display
            .tab_title_num_characters
            .map(|it| it.to_string())
            .unwrap_or_else(or_string),
    );
    println!("display.scrollbar_position = {}", cfg.display.scrollbar_position);

    // =================

    println!();

    println!("color.bg = {}", cfg.color.bg);
    println!("color.fg = {}", cfg.color.fg);
    println!("color.cursor = {}", cfg.color.cursor);
    println!("color.pallet = {}", cfg.color.pallet);
    println!(
        "color.color_00 = {}",
        cfg.color.color_00.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_01 = {}",
        cfg.color.color_01.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_02 = {}",
        cfg.color.color_02.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_03 = {}",
        cfg.color.color_03.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_04 = {}",
        cfg.color.color_04.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_05 = {}",
        cfg.color.color_05.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_06 = {}",
        cfg.color.color_06.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_07 = {}",
        cfg.color.color_07.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_08 = {}",
        cfg.color.color_08.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_09 = {}",
        cfg.color.color_09.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_10 = {}",
        cfg.color.color_10.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_11 = {}",
        cfg.color.color_11.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_12 = {}",
        cfg.color.color_12.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_13 = {}",
        cfg.color.color_13.map(|it| it.to_string()).unwrap_or_else(or_string)
    );
    println!(
        "color.color_14 = {}",
        cfg.color.color_14.map(|it| it.to_string()).unwrap_or_else(or_string)
    );

    // =================

    println!();

    println!("process.command = {}", cfg.process.command);
    println!("process.working_dir = {}", cfg.process.working_dir.unwrap_or_else(or_string));

    // =================

    println!();

    println!("keys.copy = {}", cfg.keys.copy.unwrap_or_else(or_string));
    println!("keys.paste = {}", cfg.keys.paste.unwrap_or_else(or_string));
    println!("keys.quit = {}", cfg.keys.quit.unwrap_or_else(or_string));
    println!("keys.transparency_toggle = {}", cfg.keys.transparency_toggle.unwrap_or_else(or_string));
    println!("keys.tab_add = {}", cfg.keys.tab_add.unwrap_or_else(or_string));
    println!("keys.tab_close = {}", cfg.keys.tab_close.unwrap_or_else(or_string));
    println!("keys.tab_move_backward = {}", cfg.keys.tab_move_backward.unwrap_or_else(or_string));
    println!("keys.tab_move_forward = {}", cfg.keys.tab_move_forward.unwrap_or_else(or_string));
    println!("keys.tab_goto_next = {}", cfg.keys.tab_goto_next.unwrap_or_else(or_string));
    println!("keys.tab_goto_previous = {}", cfg.keys.tab_goto_previous.unwrap_or_else(or_string));
    println!("keys.tab_goto_last = {}", cfg.keys.tab_goto_last.unwrap_or_else(or_string));
    println!("keys.tab_goto_01 = {}", cfg.keys.tab_goto_01.unwrap_or_else(or_string));
    println!("keys.tab_goto_02 = {}", cfg.keys.tab_goto_02.unwrap_or_else(or_string));
    println!("keys.tab_goto_03 = {}", cfg.keys.tab_goto_03.unwrap_or_else(or_string));
    println!("keys.tab_goto_04 = {}", cfg.keys.tab_goto_04.unwrap_or_else(or_string));
    println!("keys.tab_goto_05 = {}", cfg.keys.tab_goto_05.unwrap_or_else(or_string));
    println!("keys.tab_goto_06 = {}", cfg.keys.tab_goto_06.unwrap_or_else(or_string));
    println!("keys.tab_goto_07 = {}", cfg.keys.tab_goto_07.unwrap_or_else(or_string));
    println!("keys.tab_goto_08 = {}", cfg.keys.tab_goto_08.unwrap_or_else(or_string));

    println!("keys.font_size_inc = {}", cfg.keys.font_size_inc.unwrap_or_else(or_string));
    println!("keys.font_size_dec = {}", cfg.keys.font_size_dec.unwrap_or_else(or_string));
    println!("keys.font_size_reset = {}", cfg.keys.font_size_reset.unwrap_or_else(or_string));

    // =================

    println!();

    println!("terminal.allow_hyper_link = {}", cfg.terminal.allow_hyper_link);
    println!("terminal.audible_bell = {}", cfg.terminal.audible_bell);
    println!("terminal.cursor_blink = {}", cfg.terminal.cursor_blink);
    println!("terminal.cursor_shape = {}", cfg.terminal.cursor_shape);
    println!("terminal.scroll_on_output = {}", cfg.terminal.scroll_on_output);
    println!("terminal.scroll_on_keystroke = {}", cfg.terminal.scroll_on_keystroke);
    println!("terminal.mouse_auto_hide = {}", cfg.terminal.mouse_auto_hide);
    println!("terminal.scrollback_lines = {}", cfg.terminal.scrollback_lines);
    println!("terminal.backspace_binding = {}", cfg.terminal.backspace_binding);
    println!("terminal.delete_binding = {}", cfg.terminal.delete_binding);
    println!("terminal.word_char_exceptions = {}", cfg.terminal.word_char_exceptions);

    // =================

    println!();

    println!("behavior.terminal_exit_behavior = {}", cfg.behavior.terminal_exit_behavior);
    println!("behavior.last_tab_exit_behavior = {}", cfg.behavior.last_tab_exit_behavior);
    println!("behavior.last_tab_exit_behavior = {}", cfg.behavior.last_tab_exit_behavior);

    // =================
    #[cfg(feature = "hack")]
    {
        println!();

        println!("behavior.hack.toggle = [{}]", cfg.hack.toggle.join(", "));
    }
}

pub fn print_pallets() {
    for pallet in Pallet::all() {
        println!("\n{}: ", pallet);
        println!("[0]  = {}", pallet.colors()[0]);
        println!("[1]  = {}", pallet.colors()[1]);
        println!("[2]  = {}", pallet.colors()[2]);
        println!("[3]  = {}", pallet.colors()[3]);
        println!("[4]  = {}", pallet.colors()[4]);
        println!("[5]  = {}", pallet.colors()[5]);
        println!("[6]  = {}", pallet.colors()[6]);
        println!("[7]  = {}", pallet.colors()[7]);
        println!("[8]  = {}", pallet.colors()[8]);
        println!("[9]  = {}", pallet.colors()[9]);
        println!("[10] = {}", pallet.colors()[10]);
        println!("[11] = {}", pallet.colors()[11]);
        println!("[12] = {}", pallet.colors()[12]);
        println!("[13] = {}", pallet.colors()[13]);
        println!("[14] = {}", pallet.colors()[14]);
    }
}

pub unsafe fn macos_screens() {
    // NSWindowCollectionBehaviorCanJoinAllSpaces and friends
    let mask = 1 | (1 << 6) | (1 << 8);

    let shared: id = msg_send![class!(NSApplication), sharedApplication];
    println!("shared {:?}", shared);

    let windows: id = msg_send![shared, windows];
    println!("windows {:?}", windows);

    let window: id = msg_send![windows, firstObject];
    println!("first {:?}", window);

    if !window.is_null() {
        let _: () = msg_send![window, setCollectionBehavior:mask];
    }
    else {
        eprintln!("no window")
    }
}

