//! A callback that runs a nested event loop (a modal dialog) must not hold
//! back the callbacks queued with it: each one leaves the queue only when it
//! runs, so the nested loop runs the others. Checked for both queues,
//! `call_after_local` and `call_after`, each time with the callback that
//! opens the dialog and the one that closes it queued in the same pass.
//! Taken all together first, the second waited for the dialog to close, and
//! the dialog closed only by its fallback timer.
//!
//! Runs by itself and exits: 0 when both close their dialog from inside it,
//! 3 otherwise.

use std::cell::{Cell, RefCell};

use wxdragon::prelude::*;
use wxdragon::timer::Timer;

thread_local! {
    static SEEN: RefCell<Vec<(&'static str, usize)>> = const { RefCell::new(Vec::new()) };
    static FRAME: Cell<Option<Frame>> = const { Cell::new(None) };
    static OPEN: Cell<Option<Dialog>> = const { Cell::new(None) };
}

fn note(what: &'static str) {
    SEEN.with(|s| s.borrow_mut().push((what, open_modal_count())));
}

/// A modal dialog that a callback queued next to it must close; after 2 s it
/// closes by itself, and the check fails.
fn modal(title: &str) {
    let Some(frame) = FRAME.with(Cell::get) else {
        return;
    };
    let dialog = Dialog::builder(&frame, title).build();
    OPEN.with(|o| o.set(Some(dialog)));
    let fallback = Timer::new(&dialog);
    fallback.on_tick(move |_| {
        note("closed by the fallback timer");
        dialog.end_modal(ID_CANCEL);
    });
    fallback.start(2000, true);
    dialog.show_modal();
    drop(fallback);
    OPEN.with(|o| o.set(None));
    dialog.destroy();
}

/// Closes the open dialog from a callback.
fn close_open(what: &'static str) {
    note(what);
    if let Some(dialog) = OPEN.with(Cell::get) {
        dialog.end_modal(ID_OK);
    }
}

fn main() {
    wxdragon::main(|_| {
        let frame = Frame::builder()
            .with_title("queue and nested loop")
            .with_size(Size::new(320, 200))
            .build();
        frame.show(true);
        FRAME.with(|f| f.set(Some(frame)));
        wxdragon::call_after_local(Box::new(|| {
            modal("call_after_local");
            // The same with `call_after`: queued together in this pass.
            wxdragon::call_after(Box::new(|| {
                modal("call_after");
                wxdragon::call_after_local(Box::new(|| {
                    if let Some(frame) = FRAME.with(Cell::get) {
                        frame.destroy();
                    }
                }));
            }));
            wxdragon::call_after(Box::new(|| close_open("call_after, queued next to the dialog")));
        }));
        wxdragon::call_after_local(Box::new(|| close_open("call_after_local, queued next to the dialog")));
    })
    .unwrap();

    let seen = SEEN.with(|s| s.borrow().clone());
    let mut failed = false;
    for what in [
        "call_after_local, queued next to the dialog",
        "call_after, queued next to the dialog",
    ] {
        let got = seen.iter().find(|(w, _)| *w == what).map(|(_, n)| *n);
        // Inside the dialog: one modal open.
        let ok = got == Some(1);
        println!("{what}: open modals {got:?} {}", if ok { "ok" } else { "FAILED" });
        failed |= !ok;
    }
    let fallbacks = seen.iter().filter(|(w, _)| *w == "closed by the fallback timer").count();
    println!("closed by the fallback timer: {fallbacks} {}", if fallbacks == 0 { "ok" } else { "FAILED" });
    failed |= fallbacks > 0;
    std::process::exit(if failed { 3 } else { 0 });
}
