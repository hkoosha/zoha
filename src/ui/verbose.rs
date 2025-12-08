use crate::config::cfg::ZohaCfg;
use crate::config::color::Pallet;
use eyre::ContextCompat;
use gdk::Display;
use gdk::prelude::MonitorExt;

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

pub fn print_config(cfg: ZohaCfg) {
    let or_string = || "".to_string();

    println!("font.font = {}", cfg.font.font);

    // =================

    println!();

    println!(
        "display.monitor = {}",
        cfg.display.monitor.unwrap_or_else(or_string)
    );
    println!("display.title = {}", cfg.display.title);
    println!("display.margin_left = {}", cfg.display.margin_left);
    println!("display.margin_right = {}", cfg.display.margin_right);
    println!("display.margin_top = {}", cfg.display.margin_top);
    println!("display.margin_bottom = {}", cfg.display.margin_bottom);
    println!("display.x_pos = {}", cfg.display.x_pos);
    println!("display.y_pos = {}", cfg.display.y_pos);
    println!(
        "display.width = {}",
        cfg.display
            .width
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "display.height = {}",
        cfg.display
            .height
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "display.width_percentage = {}",
        cfg.display
            .width_percentage
            .map(|it| it.to_string())
            .unwrap_or_else(or_string),
    );
    println!(
        "display.height_percentage = {}",
        cfg.display
            .height_percentage
            .map(|it| it.to_string())
            .unwrap_or_else(or_string),
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
    println!(
        "display.scrollbar_position = {}",
        cfg.display.scrollbar_position
    );

    // =================

    println!();

    println!("color.bg = {}", cfg.color.bg);
    println!("color.fg = {}", cfg.color.fg);
    println!("color.cursor = {}", cfg.color.cursor);
    println!("color.pallet = {}", cfg.color.pallet);
    println!(
        "color.color_00 = {}",
        cfg.color
            .color_00
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_01 = {}",
        cfg.color
            .color_01
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_02 = {}",
        cfg.color
            .color_02
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_03 = {}",
        cfg.color
            .color_03
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_04 = {}",
        cfg.color
            .color_04
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_05 = {}",
        cfg.color
            .color_05
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_06 = {}",
        cfg.color
            .color_06
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_07 = {}",
        cfg.color
            .color_07
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_08 = {}",
        cfg.color
            .color_08
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_09 = {}",
        cfg.color
            .color_09
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_10 = {}",
        cfg.color
            .color_10
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_11 = {}",
        cfg.color
            .color_11
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_12 = {}",
        cfg.color
            .color_12
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_13 = {}",
        cfg.color
            .color_13
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );
    println!(
        "color.color_14 = {}",
        cfg.color
            .color_14
            .map(|it| it.to_string())
            .unwrap_or_else(or_string)
    );

    // =================

    println!();

    println!("process.command = {}", cfg.process.command);
    println!(
        "process.working_dir = {}",
        cfg.process.working_dir.unwrap_or_else(or_string)
    );

    // =================

    println!();

    println!("keys.copy = {}", cfg.keys.copy.unwrap_or_else(or_string));
    println!("keys.paste = {}", cfg.keys.paste.unwrap_or_else(or_string));
    println!("keys.quit = {}", cfg.keys.quit.unwrap_or_else(or_string));
    println!(
        "keys.transparency_toggle = {}",
        cfg.keys.transparency_toggle.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_add = {}",
        cfg.keys.tab_add.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_close = {}",
        cfg.keys.tab_close.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_move_backward = {}",
        cfg.keys.tab_move_backward.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_move_forward = {}",
        cfg.keys.tab_move_forward.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_next = {}",
        cfg.keys.tab_goto_next.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_previous = {}",
        cfg.keys.tab_goto_previous.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_last = {}",
        cfg.keys.tab_goto_last.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_01 = {}",
        cfg.keys.tab_goto_01.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_02 = {}",
        cfg.keys.tab_goto_02.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_03 = {}",
        cfg.keys.tab_goto_03.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_04 = {}",
        cfg.keys.tab_goto_04.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_05 = {}",
        cfg.keys.tab_goto_05.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_06 = {}",
        cfg.keys.tab_goto_06.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_07 = {}",
        cfg.keys.tab_goto_07.unwrap_or_else(or_string)
    );
    println!(
        "keys.tab_goto_08 = {}",
        cfg.keys.tab_goto_08.unwrap_or_else(or_string)
    );

    println!(
        "keys.font_size_inc = {}",
        cfg.keys.font_size_inc.unwrap_or_else(or_string)
    );
    println!(
        "keys.font_size_dec = {}",
        cfg.keys.font_size_dec.unwrap_or_else(or_string)
    );
    println!(
        "keys.font_size_reset = {}",
        cfg.keys.font_size_reset.unwrap_or_else(or_string)
    );

    // =================

    println!();

    println!(
        "terminal.allow_hyper_link = {}",
        cfg.terminal.allow_hyper_link
    );
    println!("terminal.audible_bell = {}", cfg.terminal.audible_bell);
    println!("terminal.cursor_blink = {}", cfg.terminal.cursor_blink);
    println!("terminal.cursor_shape = {}", cfg.terminal.cursor_shape);
    println!(
        "terminal.scroll_on_output = {}",
        cfg.terminal.scroll_on_output
    );
    println!(
        "terminal.scroll_on_keystroke = {}",
        cfg.terminal.scroll_on_keystroke
    );
    println!(
        "terminal.mouse_auto_hide = {}",
        cfg.terminal.mouse_auto_hide
    );
    println!(
        "terminal.scrollback_lines = {}",
        cfg.terminal.scrollback_lines
    );
    println!(
        "terminal.backspace_binding = {}",
        cfg.terminal.backspace_binding
    );
    println!("terminal.delete_binding = {}", cfg.terminal.delete_binding);
    println!(
        "terminal.word_char_exceptions = {}",
        cfg.terminal.word_char_exceptions
    );

    // =================

    println!();

    println!(
        "behavior.terminal_exit_behavior = {}",
        cfg.behavior.terminal_exit_behavior
    );
    println!(
        "behavior.last_tab_exit_behavior = {}",
        cfg.behavior.last_tab_exit_behavior
    );
    println!(
        "behavior.last_tab_exit_behavior = {}",
        cfg.behavior.last_tab_exit_behavior
    );

    // =================

    println!();
    println!();

    println!("style.css = {}", cfg.style.css.unwrap_or("".to_string()));
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
