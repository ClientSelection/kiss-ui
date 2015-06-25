//! A UI framework for Rust based on the KISS principle: "Keep It Simple, Stupid!"
//!
//! Built on top of the [IUP GUI library for C.][iup]
//!
//! ##Note: "valid KISS-UI context"
//! All KISS-UI static widget methods will panic if called before `kiss_ui::show_gui()` is invoked or
//! after it returns. 
//!
//! This is because the underlying IUP library has been either, respectively, not initialized yet 
//! or already deinitialized, and attempting to interact with it in either situation will likely cause
//! undefined behavior.
//!
//! ##Note: This is a (technically) leaky abstraction.
//! Because IUP only frees all its allocations when it is deinitialized, all widgets created by KISS-UI
//! will remain in-memory until `kiss_ui::show_gui()` returns. While unbounded memory growth can
//! happen with complex applications, this should not be an issue for most use-cases.
//!
//! However, some types *do* allocate large chunks of memory, or other valuable system resources, 
//! and should be manually freed when they are no longer being used. 
//! This is most evident with the `Image` struct, which can allocate large backing buffers for image data.
//!
//! All types that should be manually freed expose a `.destroy()` method which should be called
//! when they are no longer being used. This can safely be called multiple times on clones of the
//! widget types^([citation needed]).
//!
//! [iup]: http://webserver2.tecgraf.puc-rio.br/iup/

extern crate libc;
extern crate iup_sys;


macro_rules! assert_kiss_running (
    () => (
        assert!(
            ::KISS_RUNNING.load(::std::sync::atomic::Ordering::Acquire), 
            "No KISS-UI widget methods may be called before `kiss_ui::show_gui()` is invoked or after it returns!"
        )
    )
);

#[macro_use]
pub mod widget;

#[macro_use]
pub mod utils;

// Internal use modules
mod attrs;

// User-facing modules
#[macro_use]
pub mod callback;

pub mod base;
pub mod button;
pub mod container;
pub mod dialog;
pub mod image;
pub mod progress;
pub mod text;
pub mod timer;

use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};

use base::BaseWidget;
use dialog::Dialog;
use widget::Widget;

use utils::cstr::AsCStr;

use widget_prelude::IUPPtr;

mod widget_prelude {
    pub use widget::{Widget, IUPWidget, Destroy, WidgetStr};
    pub type IUPPtr = *mut ::iup_sys::Ihandle; 
}

/// A module that KISS-UI users can glob-import to get the most common types.
pub mod prelude {
    pub use base::BaseWidget;
    pub use dialog::Dialog;
    pub use container::Orientation;
    pub use callback::{CallbackStatus, OnClick, OnShow, OnValueChange};

    pub use widget::{Widget, Destroy};
}

static KISS_RUNNING: AtomicBool = ATOMIC_BOOL_INIT;

thread_local! { static CONTEXT: KISSContext = KISSContext::default() }

#[derive(Default)]
struct KISSContext {
    widget_store: RefCell<HashMap<String, BaseWidget>>,
    // FIXME: use Rc<()> once Rc::is_unique stabilizes
    borrowed_strs: RefCell<HashMap<IUPPtr, HashMap<&'static str, Rc<Cell<usize>>>>>,
}

impl KISSContext {
    fn assert_str_not_borrowed(widget: IUPPtr, str_: &'static str) {
        assert_kiss_running!();

        let is_borrowed = CONTEXT.with(|context|
            context.borrowed_strs.borrow()
                .get(&widget)
                .and_then(|widget_strs| 
                     widget_strs.get(str_)
                        .map(|refcount| refcount.get() != 0)
                )
                .unwrap_or(false)
        );

        assert!(
            !is_borrowed, 
            "Cannot update the value of a string property of a widget if it's been previously borrowed!"
        );                
    }

    fn str_refcount(widget: IUPPtr, str_: &'static str) -> Rc<Cell<usize>> {
        assert_kiss_running!();

        CONTEXT.with(|context|
            context.borrowed_strs.borrow_mut()
                .entry(widget).or_insert_with(HashMap::new)
                .entry(str_).or_insert_with(|| Rc::new(Cell::new(0)))
                .clone()
        )
    }

    fn store_widget<N: Into<String>, W: Widget>(name: N, widget: W) -> Option<BaseWidget> {
        CONTEXT.with(|context|
            context.widget_store.borrow_mut()
                .insert(name.into(), widget.to_base())
        )
    }

    fn load_widget<N: Borrow<str>>(name: &N) -> Option<BaseWidget> {
        CONTEXT.with(|context|
            context.widget_store.borrow().get(name.borrow()).cloned()
        )
    }

    unsafe fn clear() {
        CONTEXT.with(|context| {
            context.widget_store.borrow_mut().clear();
            context.borrowed_strs.borrow_mut().clear();
        })
    }
}


/// The entry point for KISS-UI. The closure argument should initialize and call `.show()`.
///
/// ##Blocks
/// Until all KISS-UI dialogs are closed.
///
/// ##Warning
/// No static widget methods from this crate may be called before this function is
/// invoked or after it returns, with the exception of the closure passed to this function.
///
/// While this function is blocked and the IUP event loop is running, any reachable code is
/// considered a "valid KISS-UI context" and may create and interact with widgets and dialogs.
///
/// After it returns, IUP is deinitialized and all static widget methods will panic to avoid
/// undefined behavior.
///
/// ##Note: `Send` bound
/// This closure will be called in the same thread where `show_gui()` is invoked. No threading is
/// involved.
/// 
/// However, without the `Send` bound it would be possible to move widget types outside
/// of the closure with safe code and interact with them after IUP has been deinitialized, 
/// which would cause undefined behavior. 
///
/// Since no widget types are `Send`, this bound prevents this from happening without requiring
/// all widget methods to check if they were invoked in a valid context.
pub fn show_gui<F>(init_fn: F) where F: FnOnce() -> Dialog + Send {
    assert!(
        !KISS_RUNNING.compare_and_swap(false, true, Ordering::SeqCst), 
        "KISS-UI may only be running (in `kiss_ui::show_gui()`) in one thread at a time!"
    );

    unsafe { 
        assert!(iup_sys::IupOpen(ptr::null(), ptr::null()) == 0);
        // Force IUP to always use UTF-8
        iup_sys::IupSetGlobal(::attrs::UTF8_MODE.as_cstr(), ::attrs::values::YES.as_cstr());
    }   

    init_fn().show();

    unsafe { 
        iup_sys::IupMainLoop();
        iup_sys::IupClose();
        KISSContext::clear();
    }

    KISS_RUNNING.store(false, Ordering::SeqCst); 
}
