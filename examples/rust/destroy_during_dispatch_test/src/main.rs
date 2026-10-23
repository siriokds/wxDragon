//! Regression test: a window destroyed while one of its own callbacks is still
//! running must not free that callback, nor the handler the dispatch is using.
//!
//! `destroy()` defers the destruction (issue #164 for child windows; top-level
//! windows wait for idle), but a callback that opens a modal dialog runs a nested
//! event loop, and the deferred destruction happens inside it, under the callback.
//!
//! Three cases, each driven by a timer whose tick destroys a window and then opens
//! a modal dialog that closes itself:
//! - `child`: a panel owns the timer and is destroyed;
//! - `ancestor`: a button inside a panel owns the timer, the panel is destroyed;
//! - `toplevel`: a second frame owns the timer and is destroyed.
//!
//! Runs by itself and exits: 0 when every case survived and its window was
//! destroyed (later, not never), 3 when a callback was freed while it was still
//! running or a window was never destroyed. For the C++ side, run it under Guard Malloc,
//! which turns any use of freed memory into an immediate crash:
//!
//! ```text
//! DYLD_INSERT_LIBRARIES=/usr/lib/libgmalloc.dylib cargo run -p destroy_during_dispatch_test
//! ```
//!
//! An argument (`child`, `ancestor`, `toplevel`) runs a single case.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

use wxdragon::prelude::*;
use wxdragon::timer::Timer;

/// The tick of the case being tried is on the stack.
static RUNNING: AtomicBool = AtomicBool::new(false);
/// The tick's closure was dropped while it was running.
static FREED_WHILE_RUNNING: AtomicBool = AtomicBool::new(false);
/// The self-closing modal dialog is open.
static IN_MODAL: AtomicBool = AtomicBool::new(false);
/// The destroyed window really went away inside the modal dialog.
static DESTROYED_IN_MODAL: AtomicBool = AtomicBool::new(false);
/// The destroyed window went away at all (now or later).
static DESTROYED: AtomicBool = AtomicBool::new(false);
static ANY_FAILED: AtomicBool = AtomicBool::new(false);

/// Captured by the tick's closure: its drop tells whether the closure was freed
/// while still running.
struct Witness;

impl Drop for Witness {
    fn drop(&mut self) {
        if RUNNING.load(Ordering::SeqCst) {
            FREED_WHILE_RUNNING.store(true, Ordering::SeqCst);
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Case {
    Child,
    Ancestor,
    TopLevel,
}

impl Case {
    fn parse(arg: &str) -> Option<Case> {
        match arg {
            "child" => Some(Case::Child),
            "ancestor" => Some(Case::Ancestor),
            "toplevel" => Some(Case::TopLevel),
            _ => None,
        }
    }
}

thread_local! {
    static FRAME: RefCell<Option<Frame>> = const { RefCell::new(None) };
    static QUEUE: RefCell<Vec<Case>> = const { RefCell::new(Vec::new()) };
    static CURRENT: RefCell<Option<Case>> = const { RefCell::new(None) };
    /// Starts the next case a little later, after the deferred destruction.
    static NEXT: RefCell<Option<Timer<Frame>>> = const { RefCell::new(None) };
}

fn main_frame() -> Frame {
    FRAME.with(|f| f.borrow().expect("the main frame"))
}

/// Notes when the destroyed window really goes away.
fn watch_destroy(window: &impl WindowEvents) {
    window.on_destroy(|event| {
        DESTROYED.store(true, Ordering::SeqCst);
        if IN_MODAL.load(Ordering::SeqCst) {
            DESTROYED_IN_MODAL.store(true, Ordering::SeqCst);
        }
        event.skip(true);
    });
}

/// A modal dialog that closes itself after a short while, so the nested event
/// loop runs (pending events and idle, where deferred destruction happens).
fn modal_that_closes_itself() {
    let frame = main_frame();
    let dialog = Dialog::builder(&frame, "destroy during dispatch").build();
    let timer = Timer::new(&dialog);
    timer.on_tick(move |_| dialog.end_modal(ID_OK));
    timer.start(300, true);
    IN_MODAL.store(true, Ordering::SeqCst);
    dialog.show_modal();
    IN_MODAL.store(false, Ordering::SeqCst);
    drop(timer);
    dialog.destroy();
}

/// Runs inside the tick, after the modal dialog: only globals, never the
/// closure's captures (they may already be freed when the bug is there).
fn case_finished() {
    RUNNING.store(false, Ordering::SeqCst);
    // The verdict comes a little later: the window must still go away once the
    // tick has returned (deferring its destruction forever would leak it).
    NEXT.with(|n| {
        if let Some(timer) = n.borrow().as_ref() {
            timer.start(200, true);
        }
    });
}

/// The verdict of the case just run.
fn verdict() {
    let Some(case) = CURRENT.with(|c| c.borrow_mut().take()) else {
        return;
    };
    let freed = FREED_WHILE_RUNNING.swap(false, Ordering::SeqCst);
    let in_modal = DESTROYED_IN_MODAL.swap(false, Ordering::SeqCst);
    let destroyed = DESTROYED.swap(false, Ordering::SeqCst);
    let verdict = if freed {
        "FAILED: callback freed while running"
    } else if !destroyed {
        "FAILED: the window was never destroyed"
    } else {
        "ok"
    };
    println!("{case:?}: destroyed {destroyed}, inside the modal {in_modal}; {verdict}");
    if freed || !destroyed {
        ANY_FAILED.store(true, Ordering::SeqCst);
    }
}

/// Starts a tick that destroys `doomed` and then opens the modal dialog. The
/// timer belongs to `owner` (which `doomed` is, or contains).
fn arm<T: WxEvtHandler + 'static>(owner: &T, doomed: impl WxWidget + Copy + 'static) {
    let witness = Witness;
    let timer = Timer::new(owner);
    timer.on_tick(move |_| {
        let _keep = &witness;
        RUNNING.store(true, Ordering::SeqCst);
        doomed.destroy();
        modal_that_closes_itself();
        case_finished();
    });
    timer.start(100, true);
    // The timer must outlive its owner here: its owner going away is the test.
    std::mem::forget(timer);
}

fn run(case: Case) {
    CURRENT.with(|c| *c.borrow_mut() = Some(case));
    let frame = main_frame();
    match case {
        Case::Child => {
            let panel = Panel::builder(&frame).build();
            watch_destroy(&panel);
            arm(&panel, panel);
        }
        Case::Ancestor => {
            let panel = Panel::builder(&frame).build();
            let button = Button::builder(&panel).with_label("inside").build();
            watch_destroy(&panel);
            arm(&button, panel);
        }
        Case::TopLevel => {
            let other = Frame::builder()
                .with_title("second frame")
                .with_size(Size::new(240, 120))
                .build();
            other.show(true);
            watch_destroy(&other);
            arm(&other, other);
        }
    }
}

fn next_case() {
    verdict();
    match QUEUE.with(|q| q.borrow_mut().pop()) {
        Some(case) => run(case),
        None => {
            NEXT.with(|n| n.borrow_mut().take());
            main_frame().destroy();
        }
    }
}

fn main() {
    let only = std::env::args().nth(1);
    let cases = match only.as_deref().map(Case::parse) {
        Some(Some(case)) => vec![case],
        Some(None) => {
            eprintln!("usage: destroy_during_dispatch_test [child|ancestor|toplevel]");
            std::process::exit(2);
        }
        None => vec![Case::Child, Case::Ancestor, Case::TopLevel],
    };
    wxdragon::main(move |_| {
        let frame = Frame::builder()
            .with_title("destroy during dispatch")
            .with_size(Size::new(320, 200))
            .build();
        frame.show(true);
        FRAME.with(|f| *f.borrow_mut() = Some(frame));
        let next = Timer::new(&frame);
        next.on_tick(|_| next_case());
        NEXT.with(|n| *n.borrow_mut() = Some(next));
        // Popped from the end: reverse so they run in the order given.
        QUEUE.with(|q| *q.borrow_mut() = cases.iter().rev().copied().collect());
        wxdragon::call_after_local(Box::new(next_case));
    })
    .unwrap();
    std::process::exit(if ANY_FAILED.load(Ordering::SeqCst) { 3 } else { 0 });
}
