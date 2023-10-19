#![allow(clippy::needless_return)]

use std::cell::RefCell;
use std::sync::Arc;

use clap::Parser;
use eyre::Result;
use gtk::Application;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;
use log::{debug, trace};

use config::cfg::read_cfg_content;
use zoha::{config, print_config, print_pallets};
use zoha::config::args::ZohaArgs;
use zoha::config::cfg::read_cfg_content;
use zoha::config::cfg::ZohaCfg;
#[cfg(target_os = "linux")]
use zoha::connect_gdk_dbus;
#[cfg(feature = "hack")]
use zoha::hack::enable_key_grabber_hack;
#[cfg(target_os = "linux")]
use zoha::list_keycodes;
use zoha::list_monitors;
use zoha::on_app_activate;
#[cfg(target_os = "linux")]
use zoha::send_toggle_signal_through_dbus;
use zoha::ui::window::create_application;
use zoha::ZohaCtx;

#[cfg(target_os = "macos")]
static mut CTX: Option<Arc<RefCell<ZohaCtx>>> = None;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args: ZohaArgs = ZohaArgs::parse();

    #[cfg(target_os = "linux")]
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

    let cfg: Arc<ZohaCfg> = Arc::new(cfg);

    let ctx: ZohaCtx = ZohaCtx::new(cfg);
    let ctx = Arc::new(RefCell::new(ctx));
    let ctx0 = Arc::clone(&ctx);

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
            Ok(_) => {
                #[cfg(target_os = "linux")]
                connect_gdk_dbus(&ctx0, app)
            }
            Err(err) => eprintln!("{}", err),
        }
    });

    unsafe {
        #[cfg(target_os = "macos")]
        if args.listener {
            debug!("enabling native key listener");
            mac_grabber::macos_key_grabber();
            CTX = Some(Arc::clone(&ctx));
        } else {
            trace!("not enabling native key listener");
            CTX = None;
        }
    }

    application.run_with_args::<String>(&[]);

    return Ok(());
}

#[cfg(target_os = "macos")]
mod mac_grabber {
    use std::ffi::c_void;
    use std::ptr;

    use libc::c_int;

    use zoha::toggle;

    #[repr(C)]
    struct EventTypeSpec {
        event_class: u32,
        event_kind: u32,
    }

    #[repr(C)]
    struct EventHotKeyID {
        signature: c_int,
        id: u32,
    }

    #[repr(C)]
    pub struct OpaqueCStruct8Hack {
        _private: [u8; 8],
    }

    #[repr(C)]
    pub struct OpaqueCStructBigHack {
        _private: [u8; 1024],
    }

    impl OpaqueCStructBigHack {
        pub fn new() -> OpaqueCStructBigHack {
            Self {
                _private: [0; 1024],
            }
        }
    }

    type Handler = extern "C" fn(
        *const c_void,
        *const c_void,
        *const c_void,
    ) -> c_int;

    extern "C" {
        fn GetApplicationEventTarget() -> OpaqueCStruct8Hack;

        fn InstallEventHandler(
            event_target: OpaqueCStruct8Hack,
            handler: Handler,
            num_events: libc::c_ulong,
            event_type_spec: *const EventTypeSpec,
            user_data: *const c_void,
            out_ref: *mut c_void,
        ) -> c_int;

        fn RegisterEventHotKey(
            key_code: u32,
            modifiers: u32,
            hotkey_id: EventHotKeyID,
            target: OpaqueCStruct8Hack,
            options: u32,
            event_hotkey_ref: *mut OpaqueCStructBigHack,
        ) -> c_int;
    }

    extern "C" fn handler(
        _next_handler: *const c_void,
        _event_ref: *const c_void,
        _user_data: *const c_void,
    ) -> c_int {
        println!("got something");
        unsafe {
            if let Some(ctx) = super::CTX.as_ref() {
                toggle(ctx);
            } else {
                eprint!("missing context on toggle");
            }
        }
        return 0;
    }

    pub unsafe fn macos_key_grabber() {
        let event_type_spec = EventTypeSpec {
            // kEventClassKeyboard
            event_class: 1801812322,

            // kEventHotKeyReleased
            event_kind: 6,
        };

        let result = InstallEventHandler(
            GetApplicationEventTarget(),
            handler,
            1,
            &event_type_spec,
            ptr::null(),
            ptr::null_mut(),
        );

        if result != 0 {
            panic!("failed to install event handler");
        }

        let key_id = EventHotKeyID {
            signature: 1,
            id: 1,
        };

        let result = RegisterEventHotKey(
            0x6D, // F10
            256, // cmd
            key_id,
            GetApplicationEventTarget(),
            0,
            &mut OpaqueCStructBigHack::new(),
        );

        if result != 0 {
            panic!("failed to register event hot key");
        }
    }
}
