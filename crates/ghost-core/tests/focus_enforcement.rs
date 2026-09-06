//! The guarantee test: under the default policy, no primitive in ghost-core can
//! touch the user's cursor, keyboard, or foreground window - and the policy
//! itself cannot be raised from inside the process while the operator's lock
//! is on.
//!
//! This is the test that keeps the product claim honest. If someone adds a new
//! `SendInput` or `SetForegroundWindow` call path and forgets the policy gate, the
//! matching assertion here fails.
//!
//! All assertions live in one test function on purpose: the focus policy is
//! process-global state, and cargo runs test functions on parallel threads.

#![cfg(windows)]

use ghost_core::error::CoreError;
use ghost_core::focus::{self, FocusPolicy};
use ghost_core::input::{keyboard, mouse};
use ghost_core::uia::tree;

fn is_blocked<T>(r: Result<T, CoreError>, what: &str) {
    match r {
        Err(CoreError::NoBackgroundPath { .. }) => {}
        Err(other) => panic!("{what} failed for the wrong reason: {other}"),
        Ok(_) => panic!("{what} was allowed to run under the Background policy"),
    }
}

#[test]
fn background_policy_blocks_every_screen_stealing_primitive() {
    focus::set_policy(FocusPolicy::Background).unwrap();
    assert!(focus::is_background_only());

    // Mouse: every one of these moves the user's real cursor.
    is_blocked(mouse::click(10, 10), "mouse::click");
    is_blocked(mouse::move_to(10, 10), "mouse::move_to");
    is_blocked(mouse::hover(10, 10), "mouse::hover");
    is_blocked(mouse::right_click(10, 10), "mouse::right_click");
    is_blocked(mouse::double_click(10, 10), "mouse::double_click");
    is_blocked(mouse::drag(10, 10, 20, 20), "mouse::drag");
    is_blocked(mouse::scroll(10, 10, "down", 1), "mouse::scroll");

    // Keyboard: these land in whatever window the user is currently typing into.
    let vk = keyboard::name_to_vk("Enter").expect("Enter is a known key");
    is_blocked(keyboard::type_text("hello"), "keyboard::type_text");
    is_blocked(keyboard::press_key(vk), "keyboard::press_key");
    is_blocked(keyboard::key_down(vk), "keyboard::key_down");
    is_blocked(keyboard::key_up(vk), "keyboard::key_up");

    // Window activation: raises a window over the user's work.
    is_blocked(tree::focus_window("Notepad"), "tree::focus_window");
    is_blocked(tree::focus_window_under_point(10, 10), "tree::focus_window_under_point");

    // The lock: with GHOST_FOCUS_LOCK unset (this process), the policy cannot be
    // raised from inside the process, so an agent cannot open the gate itself.
    assert!(focus::locked(), "the lock must be on by default");
    for p in [FocusPolicy::PreferBackground, FocusPolicy::Foreground] {
        match focus::set_policy(p) {
            Err(CoreError::FocusLocked { .. }) => {}
            other => panic!("raising the policy to {p:?} must be refused while locked, got {other:?}"),
        }
        assert!(focus::is_background_only(), "a refused change must not apply");
    }

    // Restore the default for anything else in this binary.
    focus::set_policy(FocusPolicy::Background).unwrap();
}

#[test]
fn error_messages_teach_the_route_that_needs_no_policy_change() {
    // An agent hitting either wall learns from the message alone where the work
    // can be done without the user's screen, and that only the operator can
    // hand over real input.
    let gate = focus::require_foreground_allowed("click").unwrap_err().to_string();
    assert!(gate.contains("click"), "{gate}");
    assert!(gate.contains("op=launch"), "must name the hidden-desktop route: {gate}");
    assert!(gate.contains("GHOST_FOCUS_LOCK"), "must name the operator's key: {gate}");

    let lock = CoreError::FocusLocked { requested: "foreground" }.to_string();
    assert!(lock.contains("foreground"), "{lock}");
    assert!(lock.contains("GHOST_FOCUS_LOCK"), "{lock}");
    assert!(lock.contains("op=launch"), "{lock}");
}
