// Menu definition and bindings
// (c) 2024 Ross Younger

use crate::State;
use slint::{ComponentHandle, SharedString};
use std::rc::Rc;

pub(crate) fn setup_menu(state: &Rc<State>) {
    let state_weak = Rc::downgrade(state);
    state.main_ui.on_menu_selected(move |what| {
        handle_menu(&state_weak, &what);
    });
}

fn handle_menu(state: &std::rc::Weak<State>, what: &SharedString) {
    match what.as_str() {
        "About" => do_about(state),
        "toggle-info" => toggle_info(state),
        _ => println!("Unhandled menu event {what} !!"),
    }
}

fn do_about(state: &std::rc::Weak<State>) {
    let state = state.upgrade().unwrap();
    let ui = state.main_ui.as_weak();
    let _ = ui
        .upgrade_in_event_loop(|u| {
            u.invoke_about_box();
        })
        .map_err(|err| {
            eprintln!("error: {err}");
            err
        });
}

fn toggle_info(state: &std::rc::Weak<State>) {
    let state = state.upgrade().unwrap();
    let ui = &state.main_ui;
    ui.set_info_visible(!ui.get_info_visible());
}
