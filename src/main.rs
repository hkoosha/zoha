#![allow(clippy::needless_return)]

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use clap::Parser;
use eyre::Result;
use gtk::Application;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;
use log::debug;

use config::cfg::read_cfg_content;
use zoha::{config, print_config, print_pallets};
use zoha::config::args::ZohaArgs;
use zoha::config::cfg::ZohaCfg;
use zoha::connect_gdk_dbus;
#[cfg(feature = "hack")]
use zoha::hack::enable_key_grabber_hack;
use zoha::list_keycodes;
use zoha::list_monitors;
use zoha::on_app_activate;
use zoha::send_toggle_signal_through_dbus;
use zoha::ui::window::create_application;
use zoha::ZohaCtx;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args: ZohaArgs = ZohaArgs::parse();

    if args.signal {
        send_toggle_signal_through_dbus()?;
        return Ok(());
    }

    if args.print_pallets {
        print_pallets();
        return Ok(());
    }

    #[cfg(feature = "hack")]
    if args.list_key_grabber_keys {
        let msg =
            "These keys are suitable for key grabber ONLY.\n\
             These keys are NOT suitable for other shortcuts in Zoha, as they use GTK's\n\
             specification and these keys are specific to device_query, the rust crate.";
        let att =
            "============================== ATTENTION ======================================";
        let line =
            "===============================================================================";

        println!("{}", &att);
        println!("{}", &line);
        println!("{}", &msg);
        println!("{}\n", &line);

        for k in list_keycodes() {
            println!("{}", k);
        }

        println!("{}", &att);
        println!("{}", &line);
        println!("{}", &msg);
        println!("{}", &line);
        return Ok(());
    }

    gtk::init()?;

    if args.list_monitors {
        for m in list_monitors()? {
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
        print_config(cfg);
        return Ok(());
    }

    let cfg: Rc<ZohaCfg> = Rc::new(cfg);

    let ctx: ZohaCtx = ZohaCtx::new(cfg);
    let ctx = Rc::new(RefCell::new(ctx));
    let ctx0 = Rc::clone(&ctx);

    #[cfg(feature = "hack")]
        let _guard =
        if args.keypress_grabber {
            let guard = enable_key_grabber_hack(
                args.quiet,
                &ctx.borrow().cfg.hack.toggle,
            )?;
            Some(guard)
        } else {
            if !args.quiet {
                println!(
                    "not listening for keypress, visibility can still be toggled through dbus \
                     signals or through running `{} -s`.",
                    env::args().next().unwrap_or_else(|| "zoha".to_string())
                );
            }
            None
        };

    if args.dry_run {
        println!("dry run, not launching GTK.");
        return Ok(());
    }

    let application: Application = create_application().build();

    application.connect_activate(move |app| {
        match on_app_activate(&ctx0, app) {
            Ok(_) => connect_gdk_dbus(&ctx0, app),
            Err(err) => eprintln!("{}", err),
        }
    });

    application.run_with_args::<String>(&[]);

    return Ok(());
}
