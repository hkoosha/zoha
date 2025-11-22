use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::actions::set_app_actions;
use crate::ui::actions::set_win_actions;
use crate::ui::window::add_tab;
use crate::ui::window::create_notebook;
use crate::ui::window::create_window;
use crate::ui::window::init_window;
use crate::ui::window::on_page_reorder;
use eyre::eyre;
use gdk::glib::Propagation;
use gtk::prelude::ContainerExt;
use gtk::prelude::NotebookExt;
use gtk::prelude::WidgetExt;
use gtk::Application;
use gtk::ApplicationWindow;
use crate::app::{context, signal};

pub fn on_app_activate(ctx: &Rc<RefCell<context::ZohaCtx>>, app: &Application) -> eyre::Result<()> {
    let window: ApplicationWindow = create_window(&ctx.borrow().cfg, app).build();

    if let Err(err) = init_window(&mut ctx.borrow_mut(), window) {
        if format!("{}", err) == "window already set" {
            return Err(eyre!("app already active"));
        }
    }

    let focus_ctx = Rc::clone(ctx);
    ctx.borrow()
        .get_window()
        .unwrap()
        .connect_focus_out_event(move |_, _| {
            if focus_ctx.borrow().cfg.behavior.hide_on_focus_loss {
                signal::toggle(&focus_ctx);
            }

            return Propagation::Proceed;
        });

    set_app_actions(&ctx.borrow(), app);
    set_win_actions(ctx);

    create_notebook(&mut ctx.borrow_mut());

    add_tab(ctx, !ctx.borrow().cfg.display.start_hidden);

    let reorder_ctx = Rc::clone(ctx);
    ctx.borrow()
        .get_notebook()
        .unwrap()
        .connect_page_reordered(move |_, child, index| {
            on_page_reorder(&reorder_ctx, child, index);
        });

    ctx.borrow()
        .get_window()
        .unwrap()
        .set_child(Some(ctx.borrow().get_notebook().unwrap()));

    if ctx.borrow().cfg.display.start_hidden {
        ctx.borrow().get_window().unwrap().hide();
        ctx.borrow_mut().showing = false;
    } else {
        ctx.borrow().get_window().unwrap().show_all();
    }

    return Ok(());
}
