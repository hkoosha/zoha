#![allow(clippy::needless_return)]

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use clap::Parser;
use config::cfg::read_cfg_content;
use context::ZohaCtx;
use env_logger::Env;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;
use log::debug;
use zoha::app::signal::DBusDest;
use zoha::app::window;
use zoha::app::{
    context,
    signal,
};
use zoha::config;
use zoha::config::cfg::ZohaCfg;
use zoha::err::ZohaError;
use zoha::ui::verbose;
use zoha::ui::window::{
    create_application,
    init_screen,
};

pub const GTK_APP_ID: &str = "io.koosha.zoha2";
pub const DBUS_INTERFACE: &str = "io.koosha.zoha2";
pub const DBUS_MEMBER: &str = "zoha2";
pub const DBUS_PATH: &str = "/io/koosha/zoha2";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ZohaArgs {
    /// Override location of config file.
    #[arg(short, long)]
    pub cfg_file: Option<String>,

    /// Signal Zoha to toggle visibility and exit.
    #[arg(short, long, default_value_t = false)]
    pub signal: bool,

    /// List monitors and exit.
    #[arg(long, default_value_t = false)]
    pub list_monitors: bool,

    /// Do not print hints.
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Sanitize configuration, print any errors and exit.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Print configuration and exit
    #[arg(short, long, default_value_t = false)]
    pub print_config: bool,

    /// Print color pallets
    #[arg(long, default_value_t = false)]
    pub print_pallets: bool,
}

fn main() -> Result<(), ZohaError> {
    let args = do_init()?;
    let cfg = do_read_cfg(&args)?;
    let ctx = Rc::new(RefCell::new(ZohaCtx::new(cfg.clone())));

    if !args.quiet {
        println!(
            "not listening for keypress, visibility can still be toggled through dbus \
                     signals or through running `{} -s`.",
            env::args().next().unwrap_or_else(|| "zoha".to_string())
        );
    }

    return if args.signal {
        exe_signal()
    }
    else if args.print_pallets {
        exe_print_pallets()
    }
    else if args.list_monitors {
        exe_print_monitors()
    }
    else if args.dry_run || args.print_config {
        exe_print_config(cfg)
    }
    else {
        ekran(cfg, ctx)
    };
}

const DBUS_DEST: DBusDest = DBusDest {
    interface: DBUS_INTERFACE,
    member: DBUS_MEMBER,
    path: DBUS_PATH,
};

// =============================================================================

fn do_init() -> Result<ZohaArgs, ZohaError> {
    env_logger::try_init_from_env(Env::from("ZOHA_LOG"))?;

    let args: ZohaArgs = ZohaArgs::parse();

    return Ok(args);
}

fn do_read_cfg(args: &ZohaArgs) -> Result<Rc<ZohaCfg>, ZohaError> {
    let cfg_content: String = match read_cfg_content(args.cfg_file.as_deref()) {
        Ok(config) => Ok(config),
        Err(err) => {
            if err.is_no_config() {
                debug!("no config specified, fallback to defaults");
                Ok("".to_string())
            }
            else {
                Err(err)
            }
        }
    }?;

    let cfg = ZohaCfg::from_toml(&cfg_content)?;
    let cfg = Rc::new(cfg);

    return Ok(cfg);
}

fn exe_signal() -> Result<(), ZohaError> {
    println!("signaling");
    signal::send_toggle_signal_through_dbus(&DBUS_DEST)?;
    return Ok(());
}

fn exe_print_pallets() -> Result<(), ZohaError> {
    verbose::print_pallets();
    return Ok(());
}

fn exe_print_monitors() -> Result<(), ZohaError> {
    gtk::init()?;

    for m in verbose::list_monitors()? {
        println!("{}", m);
    }

    return Ok(());
}

fn exe_print_config(cfg: Rc<ZohaCfg>) -> Result<(), ZohaError> {
    verbose::print_config(cfg);
    return Ok(());
}

fn ekran(
    cfg: Rc<ZohaCfg>,
    ctx: Rc<RefCell<ZohaCtx>>,
) -> Result<(), ZohaError> {
    gtk::init()?;

    let css = cfg.style.css.clone();

    let g_app = create_application(GTK_APP_ID).build();
    g_app.connect_activate(move |app| {
        match window::on_app_activate(&ctx, app) {
            Ok(_) => {
                signal::connect_gdk_dbus(&ctx, app, &DBUS_DEST);
                init_screen(&css);
            }
            Err(err) => eprintln!("{}", err),
        }
    });
    g_app.run_with_args::<String>(&[]);

    return Ok(());
}
