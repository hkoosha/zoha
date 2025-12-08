#![allow(clippy::needless_return)]

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use clap::Parser;
use config::cfg::read_cfg_content;
use eyre::Result;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;
use log::debug;
use zoha::app::window;
use zoha::app::{context, signal};
use zoha::config;
use zoha::config::args::ZohaArgs;
use zoha::config::cfg::ZohaCfg;
use zoha::ui::verbose;
use zoha::ui::window::{create_application, init_screen};

fn main() -> Result<()> {
    pretty_env_logger::try_init_custom_env("ZOHA_LOG").expect("could not initialize logger");

    let args: ZohaArgs = ZohaArgs::parse();

    if args.signal {
        println!("signaling");
        signal::send_toggle_signal_through_dbus()?;
        return Ok(());
    }

    if args.print_pallets {
        verbose::print_pallets();
        return Ok(());
    }

    gtk::init()?;

    if args.list_monitors {
        for m in verbose::list_monitors()? {
            println!("{}", m);
        }
        return Ok(());
    }

    let cfg_content: String = match read_cfg_content(&args) {
        Ok(config) => Ok(config),
        Err(err) => {
            if err.is_no_config() {
                debug!("no config specified, fallback to defaults");
                Ok("".to_string())
            } else {
                Err(err)
            }
        }
    }?;

    let cfg: ZohaCfg = ZohaCfg::from_toml(&cfg_content);

    if args.print_config {
        verbose::print_config(cfg);
        return Ok(());
    }

    let css = cfg.style.css.clone();
    let cfg: Rc<ZohaCfg> = Rc::new(cfg);

    let ctx = Rc::new(RefCell::new(context::ZohaCtx::new(cfg)));

    if !args.quiet {
        println!(
            "not listening for keypress, visibility can still be toggled through dbus \
                     signals or through running `{} -s`.",
            env::args().next().unwrap_or_else(|| "zoha".to_string())
        );
    }

    if args.dry_run {
        println!("dry run, not launching GTK.");
        return Ok(());
    }

    let g_app = create_application().build();
    g_app.connect_activate(move |app| match window::on_app_activate(&ctx, app) {
        Ok(_) => {
            signal::connect_gdk_dbus(&ctx, app);
            init_screen(&css);
        }
        Err(err) => eprintln!("{}", err),
    });
    g_app.run_with_args::<String>(&[]);

    return Ok(());
}
