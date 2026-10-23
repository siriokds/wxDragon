//! Checks `open_modal_count()` with modal dialogs inside each other, and that a
//! callback queued with `call_after_local` runs inside the modal event loop (the
//! reason to check the count before opening a window of its own).
//!
//! Runs by itself and exits: 0 when every count is right, 3 otherwise.

use std::cell::RefCell;

use wxdragon::prelude::*;
use wxdragon::timer::Timer;

thread_local! {
    static SEEN: RefCell<Vec<(&'static str, usize)>> = const { RefCell::new(Vec::new()) };
}

fn note(what: &'static str) {
    SEEN.with(|s| s.borrow_mut().push((what, open_modal_count())));
}

/// A modal dialog that runs `inside` from a timer of its own, then closes.
fn modal(parent: &dyn WxWidget, title: &str, inside: impl Fn() + 'static) {
    let dialog = Dialog::builder(parent, title).build();
    let timer = Timer::new(&dialog);
    timer.on_tick(move |_| {
        inside();
        dialog.end_modal(ID_OK);
    });
    timer.start(150, true);
    dialog.show_modal();
    drop(timer);
    dialog.destroy();
}

fn main() {
    wxdragon::main(|_| {
        let frame = Frame::builder()
            .with_title("open modal count")
            .with_size(Size::new(320, 200))
            .build();
        frame.show(true);
        wxdragon::call_after_local(Box::new(move || {
            note("before any dialog");
            modal(&frame, "outer", move || {
                note("in the outer dialog");
                wxdragon::call_after_local(Box::new(|| note("queued in the outer dialog")));
                modal(&frame, "inner", || note("in the inner dialog"));
                note("back in the outer dialog");
            });
            note("after both dialogs");
            frame.destroy();
        }));
    })
    .unwrap();

    let expected: [(&str, &[usize]); 6] = [
        ("before any dialog", &[0]),
        ("in the outer dialog", &[1]),
        ("in the inner dialog", &[2]),
        // Runs in whichever modal loop gets to it first: inside a dialog anyway.
        ("queued in the outer dialog", &[1, 2]),
        ("back in the outer dialog", &[1]),
        ("after both dialogs", &[0]),
    ];
    let seen = SEEN.with(|s| s.borrow().clone());
    let mut failed = false;
    for (what, allowed) in expected {
        let got = seen.iter().find(|(w, _)| *w == what).map(|(_, n)| *n);
        let ok = got.is_some_and(|n| allowed.contains(&n));
        println!("{what}: {got:?} {}", if ok { "ok" } else { "FAILED" });
        failed |= !ok;
    }
    std::process::exit(if failed { 3 } else { 0 });
}
