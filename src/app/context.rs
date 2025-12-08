use crate::config::cfg::ZohaCfg;
use crate::ui::terminal::ZohaTerminal;
use eyre::eyre;
use gtk::{ApplicationWindow, Notebook};
use log::debug;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Sub;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

struct TabCounter(usize);

pub struct ZohaCtx {
    pub(crate) cfg: Rc<ZohaCfg>,
    pub(crate) font_scale: f64,
    pub(crate) fullscreen: bool,
    pub(crate) showing: bool,
    pub(crate) terminals: Rc<RefCell<HashMap<u32, ZohaTerminal>>>,
    pub(crate) transparency_enabled: bool,
    pub(crate) last_toggle: SystemTime,
    pub(crate) x: i32,
    pub(crate) y: i32,
    tab_counter: Rc<RefCell<TabCounter>>,
    pub(crate) window: Option<ApplicationWindow>,
    notebook: Option<Notebook>,
}

impl Debug for ZohaCtx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ZohaCtx[fullscreen={}, scaling_factor={}, window={}]",
            self.fullscreen,
            self.font_scale,
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

        let x = i32::try_from(cfg.display.x_pos).expect("x_pos overflow")
            + i32::try_from(cfg.display.margin_left).expect("margin_left overflow");
        let y = i32::try_from(cfg.display.y_pos).expect("y_pos overflow")
            + i32::try_from(cfg.display.margin_top).expect("margin_top overflow");

        return Self {
            cfg,
            font_scale: 1.0,
            fullscreen,
            showing: true,
            terminals: Rc::new(RefCell::new(HashMap::new())),
            transparency_enabled: true,
            last_toggle: SystemTime::now().sub(Duration::from_secs(3600)),
            window: None,
            notebook: None,
            tab_counter: Rc::new(RefCell::new(TabCounter(1))),
            x,
            y,
        };
    }

    pub(crate) fn set_window(&mut self, window: ApplicationWindow) -> eyre::Result<()> {
        if self.window.is_some() {
            return Err(eyre!("window already set"));
        }

        debug!("setting window");
        self.window = Some(window);

        return Ok(());
    }

    pub(crate) fn set_notebook(&mut self, notebook: Notebook) {
        if self.notebook.is_some() {
            panic!("notebook already set")
        }

        debug!("setting notebook");
        self.notebook = Some(notebook);
    }

    pub(crate) fn get_window(&self) -> Option<&ApplicationWindow> {
        return self.window.as_ref();
    }

    pub(crate) fn get_notebook(&self) -> Option<&Notebook> {
        return self.notebook.as_ref();
    }

    pub(crate) fn font_inc(&mut self) {
        let old_size = self.font_scale;
        self.font_scale += 0.1;

        debug!("font scale inc {} => {}", old_size, self.font_scale);
    }

    pub(crate) fn font_dec(&mut self) {
        let old_size = self.font_scale;
        self.font_scale -= 0.1;

        debug!("font scale dec {} => {}", old_size, self.font_scale);
    }

    pub(crate) fn font_reset(&mut self) {
        let old_size = self.font_scale;
        self.font_scale = 1.0;

        debug!("font scale reset {} => {}", old_size, self.font_scale);
    }

    pub(crate) fn issue_tab_number(&self) -> usize {
        let num: usize = self.tab_counter.borrow().0;
        self.tab_counter.borrow_mut().0 += 1;
        return num;
    }
}
