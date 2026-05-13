//! Per-platform OS-snapshot occlusion (SPEC §4 / v0.2 Phase B.2).
//!
//! macOS App Switcher and Windows Task View can snapshot the visible
//! window for live thumbnails. For a GUI that handles BIP-39 mnemonics
//! and secret-class flags, those thumbnails are an exposure surface.
//! v0.2 mitigates this on macOS via `NSWindowSharingType::None` and on
//! Windows via `WDA_EXCLUDEFROMCAPTURE`. Linux has no compositor-level
//! analogue at v0.2 and remains documented at FOLLOWUPS
//! `gui-os-snapshot-secret-occlusion`.
//!
//! This is the first module in the codebase containing `unsafe`. Every
//! `unsafe` block carries a `// SAFETY:` comment.

use raw_window_handle::{RawWindowHandle, WindowHandle};

/// Apply OS-level window-capture protection. Idempotent;
/// non-fatal on failure (the GUI runs fine without the protection,
/// just with the documented exposure surface).
///
/// SPEC §4 R1 C-2 fold: takes the borrow-guarded `WindowHandle<'_>`
/// rather than the inner `RawWindowHandle` enum. The borrow guards the
/// validity of the underlying platform handle for the call duration;
/// `RawWindowHandle` is `#[non_exhaustive]` and `!Send`, so taking the
/// inner enum directly would discard the lifetime guarantee that rwh
/// 0.6 was specifically designed to provide.
pub fn apply_window_capture_protection(handle: WindowHandle<'_>) {
    let raw = handle.as_raw();
    apply_for_raw(raw);
}

#[cfg(target_os = "macos")]
fn apply_for_raw(raw: RawWindowHandle) {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSView, NSWindow, NSWindowSharingType};

    let RawWindowHandle::AppKit(h) = raw else {
        tracing::warn!(
            "OS-snapshot occlusion (macOS): expected AppKit handle, got {:?}; skip",
            std::mem::discriminant(&raw)
        );
        return;
    };

    // SAFETY: `h.ns_view` is a valid NSView pointer for the duration of
    // the WindowHandle borrow. winit/eframe constructs this from an
    // already-retained NSView. We immediately convert to a typed
    // reference and immediately query `.window()` — no long-lived alias.
    let view: &NSView = unsafe {
        let ptr = h.ns_view.as_ptr().cast::<NSView>();
        // ns_view is `NonNull<c_void>` — cast then deref.
        &*ptr
    };

    // SAFETY: view is a live NSView; `.window()` is a documented
    // accessor returning the parent NSWindow (or nil if the view is not
    // yet attached). Both safe to call.
    let window: Option<Retained<NSWindow>> = unsafe { view.window() };

    let Some(window) = window else {
        tracing::warn!("OS-snapshot occlusion (macOS): NSView has no parent NSWindow yet; skip");
        return;
    };

    // SAFETY: window is a valid retained NSWindow. setSharingType is a
    // documented method on NSWindow (NSWindowSharingType::None = 0).
    unsafe {
        window.setSharingType(NSWindowSharingType::None);
    }

    // Silence unused-import warnings on architecture mismatches.
    let _ = std::any::type_name::<AnyObject>();
    tracing::debug!(
        "OS-snapshot occlusion (macOS): NSWindowSharingType::None applied"
    );
}

#[cfg(windows)]
fn apply_for_raw(raw: RawWindowHandle) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
    };

    let RawWindowHandle::Win32(h) = raw else {
        tracing::warn!(
            "OS-snapshot occlusion (Windows): expected Win32 handle, got {:?}; skip",
            std::mem::discriminant(&raw)
        );
        return;
    };

    let hwnd = HWND(h.hwnd.get() as *mut std::ffi::c_void);

    // SAFETY: `hwnd` is the live HWND from a valid WindowHandle borrow.
    // SetWindowDisplayAffinity is documented as safe to call on any
    // valid HWND; a failure result is non-fatal — we log it and return.
    let result = unsafe { SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) };

    match result {
        Ok(()) => {
            tracing::debug!(
                "OS-snapshot occlusion (Windows): WDA_EXCLUDEFROMCAPTURE applied"
            );
        }
        Err(e) => {
            tracing::warn!(
                "OS-snapshot occlusion (Windows): SetWindowDisplayAffinity failed: {e}"
            );
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
fn apply_for_raw(_raw: RawWindowHandle) {
    tracing::debug!(
        "OS-snapshot occlusion: no compositor API on this platform \
         (Linux/BSD/other); see FOLLOWUPS gui-os-snapshot-secret-occlusion"
    );
}

/// Used by `tests/schema_mirror.rs::platform_module_compiles_linux` to
/// assert the module signature is reachable. The test does not actually
/// invoke the function (no live `WindowHandle` available in a unit
/// test).
#[doc(hidden)]
pub fn _signature_check() -> fn(WindowHandle<'_>) {
    apply_window_capture_protection
}
