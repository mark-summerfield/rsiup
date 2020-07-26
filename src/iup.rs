// Copyright © 2020 Mark Summerfield. All rights reserved.
// Licensed under the Apache License, Version 2.0.

use crate::prelude::*;
use crate::{xerr, xerror::{xerror, XResult}};
use lazy_static::lazy_static;
use libloading::{Library, Symbol};
use scopeguard;
use std::env;
use std::path::PathBuf;
use std::ptr;
use std::str;

lazy_static! {
    pub(crate) static ref IUP_LIB: Library = Library::new(iup_dll()).expect(
        "Failed to find IUP library");
    pub(crate) static ref IM_LIB: Library =
        with_env("LD_LIBRARY_PATH", exe_path().to_str().unwrap(), || {
            Library::new(im_dll()).expect("Failed to find IM library")
        });
    pub static ref IUP: Iup<'static> = Iup::new().expect(
        "Failed to create IUP object");
    pub static ref IM: Im<'static> = Im::new();
}

fn iup_dll() -> PathBuf {
    exe_path().join(if cfg!(windows) { "iup.dll" } else { "libiup.so" })
}

fn im_dll() -> PathBuf {
    exe_path().join(if cfg!(windows) { "iupim.dll" } else { "libiupim.so" })
}

fn exe_path() -> PathBuf {
    let exe = env::current_exe().expect("Failed to find exe's path");
    let root = exe.parent().expect(
        "Failed to find location of IUP library");
    let mut root = root.to_path_buf();
    if cfg!(windows) {
        root.push("iup/windows");
    } else {
        root.push("iup/linux");
    }
    root
}

fn with_env<R> (key: impl AsRef<::std::ffi::OsStr>,
                value: impl AsRef<::std::ffi::OsStr>,
                func: impl FnOnce() -> R) -> R {
    let key = key.as_ref();
    let prev = env::var_os(key);
    env::set_var(key, value);
    scopeguard::defer!(if let Some(prev) = prev {
        env::set_var(key, prev);
    });
    func()
}

pub fn set_library_path () {
#[cfg(unix)] {
    use ::std::{
        env,
        ffi::OsStr,
        ops::Not,
        os::unix::{
            ffi::OsStrExt,
            process::ExitStatusExt,
        },
    };
    if env::var("__RECURSION_HACK__").map_or(false, |s| s == "1").not() {
        ::std::process::exit({
            let exe = env::current_exe().unwrap();
            let exe_dir = exe_path();
            let mut library_path = exe_dir.as_os_str();
            let mut storage: Vec<u8>;
            if let Some(ref os_str) = ::std::env::var_os(
                    "LD_LIBRARY_PATH") {
                storage = library_path.as_bytes().to_vec();
                storage.push(b':');
                storage.extend_from_slice(os_str.as_bytes());
                library_path = OsStr::from_bytes(&storage);
            }
            let status =
                ::std::process::Command::new(exe)
                    .env("__RECURSION_HACK__", "1")
                    .args(&env::args_os().collect::<Vec<_>>())
                    .env("LD_LIBRARY_PATH", library_path)
                    .status()
                    .expect("Failed to re-execute itself")
            ;
            match status.code() {
                | Some(exit_code) => exit_code,
                | None => {
                    // Terminated by a signal
                    let signal = status.signal().unwrap(); 
                    panic!("Process terminated with signal {}", signal);
                },
            }
        });
    }
}}

pub struct Im<'a> { // TODO move to im.rs
    _loadimage: Symbol<'a, SigCrH>,
}

impl<'a> Im<'a> {
    fn new() -> Im<'a> {
        Im {
            _loadimage: unsafe { IM_LIB.get(b"IupLoadImage\0").unwrap() },
        }
    }

    pub fn load_image(&self, name: &str) -> *mut Ihandle {
        (self._loadimage)(c_from_str(&name))
    }
}

pub struct Iup<'a> {
    _append: Symbol<'a, SigHHrH>,
    _button: Symbol<'a, SigCCrH>,
    _close: Symbol<'a, SigVrV>,
    _dialog: Symbol<'a, SigHrH>,
    _getattribute: Symbol<'a, SigHCrC>,
    _getattributeih: Symbol<'a, SigHCrH>,
    _getdialogchild: Symbol<'a, SigHCrH>,
    _getglobal: Symbol<'a, SigCrC>,
    _getint: Symbol<'a, SigHCrI>,
    _hbox: Symbol<'a, SigHsrH>,
    _label: Symbol<'a, SigCrH>,
    _mainloop: Symbol<'a, SigVrI>,
    _message: Symbol<'a, SigCCrV>,
    _setattribute: Symbol<'a, SigHCCrV>,
    _setattributehandle: Symbol<'a, SigHCHrV>,
    _setattributeih: Symbol<'a, SigHCHrV>,
    _setcallback: Symbol<'a, SigHCKrK>,
    _setfocus: Symbol<'a, SigHrH>,
    _setglobal: Symbol<'a, SigCCrV>,
    _sethandle: Symbol<'a, SigCHrH>,
    _setint: Symbol<'a, SigHCIrV>,
    _show: Symbol<'a, SigHrI>,
    _showxy: Symbol<'a, SigHIIrI>,
    _timer: Symbol<'a, SigVrH>,
    _vbox: Symbol<'a, SigHsrH>,
    _version: Symbol<'a, SigVrC>,
    _versionshow: Symbol<'a, SigVrV>,
}

impl<'a> Iup<'a> {
    fn new() -> XResult<Iup<'a>> {
        let iup_open: Symbol<SigpIpppCrI> = unsafe {
            IUP_LIB.get(b"IupOpen\0").unwrap()
        };
        if iup_open(ptr::null(), ptr::null()) != NOERROR {
            xerr!("Failed to open IUP library");
        }
        let setglobal: Symbol<SigCCrV> = unsafe {
            IUP_LIB.get(b"IupSetGlobal\0").unwrap()
        };
        setglobal(c_from_str(UTF8MODE), c_from_str(YES));
        Ok(Iup {
            _append: unsafe { IUP_LIB.get(b"IupAppend\0").unwrap() },
            _button: unsafe { IUP_LIB.get(b"IupButton\0").unwrap() },
            _close: unsafe { IUP_LIB.get(b"IupClose\0").unwrap() },
            _dialog: unsafe { IUP_LIB.get(b"IupDialog\0").unwrap() },
            _getattribute: unsafe {
                IUP_LIB.get(b"IupGetAttribute\0").unwrap() },
            _getattributeih: unsafe {
                IUP_LIB.get(b"IupGetAttribute\0").unwrap() },
            _getdialogchild: unsafe {
                IUP_LIB.get(b"IupGetDialog\0").unwrap() },
            _getglobal: unsafe { IUP_LIB.get(b"IupGetGlobal\0").unwrap() },
            _getint: unsafe { IUP_LIB.get(b"IupGetInt\0").unwrap() },
            _hbox: unsafe { IUP_LIB.get(b"IupHbox\0").unwrap() },
            _label: unsafe { IUP_LIB.get(b"IupLabel\0").unwrap() },
            _mainloop: unsafe { IUP_LIB.get(b"IupMainLoop\0").unwrap() },
            _message: unsafe { IUP_LIB.get(b"IupMessage\0").unwrap() },
            _setattribute: unsafe {
                IUP_LIB.get(b"IupSetAttribute\0").unwrap() },
            _setattributehandle: unsafe {
                IUP_LIB.get(b"IupSetAttributeHandle\0").unwrap() },
            _setattributeih: unsafe {
                IUP_LIB.get(b"IupSetAttribute\0").unwrap() },
            _setcallback: unsafe {
                IUP_LIB.get(b"IupSetCallback\0").unwrap() },
            _setfocus: unsafe { IUP_LIB.get(b"IupSetFocus\0").unwrap() },
            _setglobal: setglobal,
            _sethandle: unsafe { IUP_LIB.get(b"IupSetHandle\0").unwrap() },
            _setint: unsafe { IUP_LIB.get(b"IupSetInt\0").unwrap() },
            _show: unsafe { IUP_LIB.get(b"IupShow\0").unwrap() },
            _showxy: unsafe { IUP_LIB.get(b"IupShowXY\0").unwrap() },
            _timer: unsafe { IUP_LIB.get(b"IupTimer\0").unwrap() },
            _vbox: unsafe { IUP_LIB.get(b"IupVbox\0").unwrap() },
            _version: unsafe { IUP_LIB.get(b"IupVersion\0").unwrap() },
            _versionshow: unsafe {
                IUP_LIB.get(b"IupVersionShow\0").unwrap() },
        })
    }
    
    pub fn append(&self, ih: *mut Ihandle,
                  child: *mut Ihandle) -> *mut Ihandle {
        (self._append)(ih, child)
    }

    pub fn button(&self, title: &str, action: &str) -> *mut Ihandle {
        (self._button)(c_from_str(&title), c_from_str(&action))
    }

    pub fn close(&self) { // MUST be called ONCE at termination
        (self._close)()
    }

    pub fn dialog(&self, child: *mut Ihandle) -> *mut Ihandle {
        (self._dialog)(child)
    }

    pub fn get_attribute(&self, ih: *mut Ihandle,
                         name: &str) -> Option<String> {
        match c_to_string((self._getattribute)(ih, c_from_str(&name))) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    pub fn get_dialog_child(&self, ih: *mut Ihandle,
                            name: &str) -> *mut Ihandle {
        (self._getdialogchild)(ih, c_from_str(&name))
    }

    pub fn get_global(&self, name: &str) -> String {
        match c_to_string((self._getglobal)(c_from_str(name))) {
            Ok(v) => v,
            Err(_) => "".to_string(),
        }
    }

    pub fn get_ih(&self, ih: *mut Ihandle, name: &str) -> *mut Ihandle {
        (self._getattributeih)(ih, c_from_str(&name)) as *mut Ihandle
    }

    pub fn get_int(&self, ih: *mut Ihandle, name: &str) -> i32 {
        (self._getint)(ih, c_from_str(&name))
    }

    pub fn hbox(&self) -> *mut Ihandle {
        (self._hbox)(self.null_ihandle()) // We always create it empty
    }

    pub fn label(&self, title: &str) -> *mut Ihandle {
        (self._label)(c_from_str(&title))
    }

    pub fn main_loop(&self) { // MUST only be called ONCE
        (self._mainloop)(); // Always returns NOERROR
    }

    pub fn message(&self, title: &str, message: &str) {
        (self._message)(c_from_str(&title), c_from_str(&message));
    }

    pub fn null_ihandle(&self) -> *mut Ihandle {
        let ih: *mut Ihandle = ptr::null_mut();
        ih
    }

    pub fn set_attribute(&self, ih: *mut Ihandle, name: &str, value: &str) {
        (self._setattribute)(ih, c_from_str(&name), c_from_str(&value));
    }

    pub fn set_attribute_handle(&self, ih: *mut Ihandle, name: &str,
                                ih_named: *mut Ihandle) {
        (self._setattributehandle)(ih, c_from_str(&name), ih_named);
    }

    pub fn set_callback(&self, ih: *mut Ihandle, name: &str,
                        func: Icallback) -> Icallback {
        (self._setcallback)(ih, c_from_str(&name), func)
    }

    pub fn set_focus(&self, ih: *mut Ihandle) -> *mut Ihandle {
        (self._setfocus)(ih)
    }

    pub fn set_global(&self, name: &str, value: &str) {
        (self._setglobal)(c_from_str(&name), c_from_str(&value));
    }

    pub fn set_handle(&self, name: &str, ih: *mut Ihandle) -> *mut Ihandle {
        (self._sethandle)(c_from_str(&name), ih)
    }

    pub fn set_ih(&self, ih: *mut Ihandle, name: &str, ihx: *mut Ihandle) {
        (self._setattributeih)(ih, c_from_str(&name), ihx);
    }

    pub fn set_int(&self, ih: *mut Ihandle, name: &str, value: i32) {
        (self._setint)(ih, c_from_str(&name), value);
    }

    pub fn show(&self, ih: *mut Ihandle) -> bool {
        (self._show)(ih) == NOERROR
    }

    pub fn show_xy(&self, ih: *mut Ihandle, x: i32, y: i32) -> bool {
        (self._showxy)(ih, x, y) == NOERROR
    }

    pub fn timer(&self) -> *mut Ihandle {
        (self._timer)()
    }

    pub fn vbox(&self) -> *mut Ihandle {
        (self._vbox)(self.null_ihandle()) // We always create it empty
    }

    pub fn version(&self) -> String {
        match c_to_string((self._version)()) {
            Ok(v) => v,
            Err(_) => "0.0".to_string(),
        }
    }

    pub fn version_show(&self) {
        (self._versionshow)();
    }
}

pub(crate) type SigCCrH = extern "C" fn(*const i8, *const i8) -> *mut Ihandle;
pub(crate) type SigCCrV = extern "C" fn(*const i8, *const i8);
pub(crate) type SigCHrH = extern "C" fn(*const i8, *mut Ihandle) -> *mut Ihandle;
pub(crate) type SigCrC = extern "C" fn(*const i8) -> *const i8;
pub(crate) type SigCrH = extern "C" fn(*const i8) -> *mut Ihandle;
pub(crate) type SigHCCrV = extern "C" fn(*mut Ihandle, *const i8, *const i8);
pub(crate) type SigHCHrV = extern "C" fn(*mut Ihandle, *const i8, *mut Ihandle);
pub(crate) type SigHCIrV = extern "C" fn(*mut Ihandle, *const i8, i32);
pub(crate) type SigHCKrK = extern "C" fn(*mut Ihandle, *const i8, Icallback) -> Icallback;
pub(crate) type SigHCrC = extern "C" fn(*mut Ihandle, *const i8) -> *const i8;
pub(crate) type SigHCrH = extern "C" fn(*mut Ihandle, *const i8) -> *mut Ihandle;
pub(crate) type SigHCrI = extern "C" fn(*mut Ihandle, *const i8) -> i32;
pub(crate) type SigHHrH = extern "C" fn(*mut Ihandle, *mut Ihandle) -> *mut Ihandle;
pub(crate) type SigHIIrI = extern "C" fn(*mut Ihandle, i32, i32) -> i32;
pub(crate) type SigHrH = extern "C" fn(*mut Ihandle) -> *mut Ihandle;
pub(crate) type SigHrI = extern "C" fn(*mut Ihandle) -> i32;
pub(crate) type SigHsrH = extern "C" fn(*mut Ihandle, ...) -> *mut Ihandle;
pub(crate) type SigVrC = extern "C" fn() -> *const i8;
pub(crate) type SigVrH = extern "C" fn() -> *mut Ihandle;
pub(crate) type SigVrI = extern "C" fn() -> i32;
pub(crate) type SigVrV = extern "C" fn();
pub(crate) type SigpIpppCrI = extern "C" fn(*const i32, *const *const *const i8) -> i32;

/*

////////
// #include "iupkey.h"
// #include "iupdef.h"
////////
// #include "iup.h"
use std::os::raw::{c_uchar, c_int, c_float, c_double, c_void};

pub const IUP_NAME: &'static str         = "IUP - Portable User Interface";
pub const IUP_DESCRIPTION: &'static str  = "Multi-platform Toolkit for Building Graphical User Interfaces";
pub const IUP_COPYRIGHT: &'static str    = "Copyright (C) 1994-2020 Tecgraf/PUC-Rio";
pub const IUP_VERSION: &'static str      = "3.29"; // bug fixes are reported only by IupVersion functions 
pub const IUP_VERSION_NUMBER: c_int      = 329000;
pub const IUP_VERSION_DATE: &'static str = "2020/05/18"; // does not include bug fix releases 

pub enum Ihandle {}
pub type Icallback = extern fn(ih: *mut Ihandle) -> c_int;
pub type Iparamcb = extern fn (dialog: *mut Ihandle, param_index: c_int, user_data: *mut c_void) -> c_int;

extern {
    ////////
    //                        Main API                                      
    ////////
    pub fn IupOpen(argc: *const c_int, argv: *const *const *const c_char) -> c_int;
    pub fn IupClose();
    #[cfg(not(any(v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_28
    pub fn IupIsOpened() -> c_int;

    pub fn IupImageLibOpen();

    pub fn IupMainLoop() -> c_int;
    pub fn IupLoopStep() -> c_int;
    pub fn IupLoopStepWait() -> c_int;
    pub fn IupMainLoopLevel() -> c_int;
    pub fn IupFlush();
    pub fn IupExitLoop();
    #[cfg(not(any(v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_28
    pub fn IupPostMessage(ih_addressee: *mut Ihandle, s: *const c_char, i: c_int, d: c_double, p: *mut c_void);

    pub fn IupRecordInput(filename: *const c_char, mode: c_int) -> c_int;
    pub fn IupPlayInput(filename: *const c_char) -> c_int;

    pub fn IupUpdate(ih: *mut Ihandle);
    pub fn IupUpdateChildren(ih: *mut Ihandle);
    pub fn IupRedraw(ih: *mut Ihandle, children: c_int);
    pub fn IupRefresh(ih: *mut Ihandle);
    pub fn IupRefreshChildren(ih: *mut Ihandle);

    #[cfg(not(any(v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_17
    pub fn IupExecute(filename: *const c_char, parameters: *const c_char) -> c_int;
    #[cfg(not(any(v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_20
    pub fn IupExecuteWait(filename: *const c_char, parameters: *const c_char) -> c_int;
    pub fn IupHelp(url: *const c_char) -> c_int;
    #[cfg(not(any(v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_23
    pub fn IupLog(r#type: *const c_char, format: *const c_char, ...);

    pub fn IupLoad(filename: *const c_char) -> *mut c_char;
    pub fn IupLoadBuffer(buffer: *const c_char) -> *mut c_char;

    pub fn IupVersion() -> *mut c_char;
    pub fn IupVersionDate() -> *mut c_char;
    pub fn IupVersionNumber() -> c_int;
    #[cfg(not(any(v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_28
    pub fn IupVersionShow();

    pub fn IupSetLanguage(lng: *const c_char);
    pub fn IupGetLanguage() -> *mut c_char;
    pub fn IupSetLanguageString(name: *const c_char, str: *const c_char);
    pub fn IupStoreLanguageString(name: *const c_char, str: *const c_char);
    pub fn IupGetLanguageString(name: *const c_char) -> *mut c_char;
    pub fn IupSetLanguagePack(ih: *mut Ihandle);

    pub fn IupDestroy(ih: *mut Ihandle);
    pub fn IupDetach(child: *mut Ihandle);
    pub fn IupAppend(ih: *mut Ihandle, child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupInsert(ih: *mut Ihandle, ref_child: *mut Ihandle, child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGetChild(ih: *mut Ihandle, pos: c_int) -> *mut Ihandle;
    pub fn IupGetChildPos(ih: *mut Ihandle, child: *mut Ihandle) -> c_int;
    pub fn IupGetChildCount(ih: *mut Ihandle) -> c_int;
    pub fn IupGetNextChild(ih: *mut Ihandle, child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGetBrother(ih: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGetParent(ih: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGetDialog(ih: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGetDialogChild(ih: *mut Ihandle, name: *const c_char) -> *mut Ihandle;
    pub fn IupReparent(ih: *mut Ihandle, new_parent: *mut Ihandle, ref_child: *mut Ihandle) -> c_int;

    pub fn IupPopup(ih: *mut Ihandle, x: c_int, y: c_int) -> c_int;
    pub fn IupShow(ih: *mut Ihandle) -> c_int;
    pub fn IupShowXY(ih: *mut Ihandle, x: c_int, y: c_int) -> c_int;
    pub fn IupHide(ih: *mut Ihandle) -> c_int;
    pub fn IupMap(ih: *mut Ihandle) -> c_int;
    pub fn IupUnmap(ih: *mut Ihandle);

    pub fn IupResetAttribute(ih: *mut Ihandle, name: *const c_char);
    pub fn IupGetAllAttributes(ih: *mut Ihandle, names: *mut *mut c_char, n: c_int) -> c_int;
    #[cfg(not(any(v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_26
    pub fn IupCopyAttributes(src_ih: *mut Ihandle, dst_ih: *mut Ihandle);
    pub fn IupSetAtt(handle_name: *const c_char, ih: *mut Ihandle, name: *const c_char, ...) -> *mut Ihandle;
    pub fn IupSetAttributes(ih: *mut Ihandle, str: *const c_char) -> *mut Ihandle;
    pub fn IupGetAttributes(ih: *mut Ihandle) -> *mut c_char;

    pub fn IupSetAttribute(ih: *mut Ihandle, name: *const c_char, value: *const c_char);
    pub fn IupSetStrAttribute(ih: *mut Ihandle, name: *const c_char, value: *const c_char);
    pub fn IupSetStrf(ih: *mut Ihandle, name: *const c_char, format: *const c_char, ...);
    pub fn IupSetInt(ih: *mut Ihandle, name: *const c_char, value: c_int);
    pub fn IupSetFloat(ih: *mut Ihandle, name: *const c_char, value: c_float);
    pub fn IupSetDouble(ih: *mut Ihandle, name: *const c_char, value: c_double);
    pub fn IupSetRGB(ih: *mut Ihandle, name: *const c_char, r: c_uchar, g: c_uchar, b: c_uchar);
    #[cfg(not(any(v3_28, v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_29
    pub fn IupSetRGBA(ih: *mut Ihandle, name: *const c_char, r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar);

    pub fn IupGetAttribute(ih: *mut Ihandle, name: *const c_char) -> *mut c_char;
    pub fn IupGetInt(ih: *mut Ihandle, name: *const c_char) -> c_int;
    pub fn IupGetInt2(ih: *mut Ihandle, name: *const c_char) -> c_int;
    pub fn IupGetIntInt(ih: *mut Ihandle, name: *const c_char, i1: *mut c_int, i2: *mut c_int) -> c_int;
    pub fn IupGetFloat(ih: *mut Ihandle, name: *const c_char) -> c_float;
    pub fn IupGetDouble(ih: *mut Ihandle, name: *const c_char) -> c_double;
    pub fn IupGetRGB(ih: *mut Ihandle, name: *const c_char, r: *mut c_uchar, g: *mut c_uchar, b: *mut c_uchar);
    #[cfg(not(any(v3_28, v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_29
    pub fn IupGetRGBA(ih: *mut Ihandle, name: *const c_char, r: *mut c_uchar, g: *mut c_uchar, b: *mut c_uchar, a: *mut c_uchar);

    pub fn IupSetAttributeId(ih: *mut Ihandle, name: *const c_char, id: c_int, value: *const c_char);
    pub fn IupSetStrAttributeId(ih: *mut Ihandle, name: *const c_char, id: c_int, value: *const c_char);
    pub fn IupSetStrfId(ih: *mut Ihandle, name: *const c_char, id: c_int, format: *const c_char, ...);
    pub fn IupSetIntId(ih: *mut Ihandle, name: *const c_char, id: c_int, value: c_int);
    pub fn IupSetFloatId(ih: *mut Ihandle, name: *const c_char, id: c_int, value: c_float);
    pub fn IupSetDoubleId(ih: *mut Ihandle, name: *const c_char, id: c_int, value: c_double);
    pub fn IupSetRGBId(ih: *mut Ihandle, name: *const c_char, id: c_int, r: c_uchar, g: c_uchar, b: c_uchar);

    pub fn IupGetAttributeId(ih: *mut Ihandle, name: *const c_char, id: c_int) -> *mut c_char;
    pub fn IupGetIntId(ih: *mut Ihandle, name: *const c_char, id: c_int) -> c_int;
    pub fn IupGetFloatId(ih: *mut Ihandle, name: *const c_char, id: c_int) -> c_float;
    pub fn IupGetDoubleId(ih: *mut Ihandle, name: *const c_char, id: c_int) -> c_double;
    pub fn IupGetRGBId(ih: *mut Ihandle, name: *const c_char, id: c_int, r: *mut c_uchar, g: *mut c_uchar, b: *mut c_uchar);

    pub fn IupSetAttributeId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, value: *const c_char);
    pub fn IupSetStrAttributeId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, value: *const c_char);
    pub fn IupSetStrfId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, format: *const c_char, ...);
    pub fn IupSetIntId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, value: c_int);
    pub fn IupSetFloatId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, value: c_float);
    pub fn IupSetDoubleId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, value: c_double);
    pub fn IupSetRGBId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, r: c_uchar, g: c_uchar, b: c_uchar);

    pub fn IupGetAttributeId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int) -> *mut c_char;
    pub fn IupGetIntId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int) -> c_int;
    pub fn IupGetFloatId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int) -> c_float;
    pub fn IupGetDoubleId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int) -> c_double;
    pub fn IupGetRGBId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, r: *mut c_uchar, g: *mut c_uchar, b: *mut c_uchar);

    pub fn IupSetGlobal(name: *const c_char, value: *const c_char);
    pub fn IupSetStrGlobal(name: *const c_char, value: *const c_char);
    pub fn IupGetGlobal(name: *const c_char) -> *mut c_char;

    pub fn IupSetFocus(ih: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGetFocus() -> *mut Ihandle;
    pub fn IupPreviousField(ih: *mut Ihandle) -> *mut Ihandle;
    pub fn IupNextField(ih: *mut Ihandle) -> *mut Ihandle;

    pub fn IupGetCallback(ih: *mut Ihandle, name: *const c_char) -> Icallback;
    pub fn IupSetCallback(ih: *mut Ihandle, name: *const c_char, func: Icallback) -> Icallback;
    pub fn IupSetCallbacks(ih: *mut Ihandle, name: *const c_char, func: Icallback, ...) -> *mut Ihandle;

    pub fn IupGetFunction(name: *const c_char) -> Icallback;
    pub fn IupSetFunction(name: *const c_char, func: Icallback) -> Icallback;

    pub fn IupGetHandle(name: *const c_char) -> *mut Ihandle;
    pub fn IupSetHandle(name: *const c_char, ih: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGetAllNames(names: *mut *mut c_char, n: c_int) -> c_int;
    pub fn IupGetAllDialogs(names: *mut *mut c_char, n: c_int) -> c_int;
    pub fn IupGetName(ih: *mut Ihandle) -> *mut c_char;

    pub fn IupSetAttributeHandle(ih: *mut Ihandle, name: *const c_char, ih_named: *mut Ihandle);
    pub fn IupGetAttributeHandle(ih: *mut Ihandle, name: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_21
    pub fn IupSetAttributeHandleId(ih: *mut Ihandle, name: *const c_char, id: c_int, ih_named: *mut Ihandle);
    #[cfg(not(any(v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_21
    pub fn IupGetAttributeHandleId(ih: *mut Ihandle, name: *const c_char, id: c_int) -> *mut Ihandle;
    #[cfg(not(any(v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_21
    pub fn IupSetAttributeHandleId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int, ih_named: *mut Ihandle);
    #[cfg(not(any(v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_21
    pub fn IupGetAttributeHandleId2(ih: *mut Ihandle, name: *const c_char, lin: c_int, col: c_int) -> *mut Ihandle;

    pub fn IupGetClassName(ih: *mut Ihandle) -> *mut c_char;
    pub fn IupGetClassType(ih: *mut Ihandle) -> *mut c_char;
    pub fn IupGetAllClasses(names: *mut *mut c_char, n: c_int) -> c_int;
    pub fn IupGetClassAttributes(classname: *const c_char, names: *mut *mut c_char, n: c_int) -> c_int;
    pub fn IupGetClassCallbacks(classname: *const c_char, names: *mut *mut c_char, n: c_int) -> c_int;
    pub fn IupSaveClassAttributes(ih: *mut Ihandle);
    pub fn IupCopyClassAttributes(src_ih: *mut Ihandle, dst_ih: *mut Ihandle);
    pub fn IupSetClassDefaultAttribute(classname: *const c_char, name: *const c_char, value: *const c_char);
    pub fn IupClassMatch(ih: *mut Ihandle, classname: *const c_char) -> c_int;

    pub fn IupCreate(classname: *const c_char) -> *mut Ihandle;
    pub fn IupCreatev(classname: *const c_char, params: *mut *mut c_void) -> *mut Ihandle;
    pub fn IupCreatep(classname: *const c_char, first: *mut c_void, ...) -> *mut Ihandle;

    ////////
    //                        Elements                                      
    ////////
    pub fn IupFill() -> *mut Ihandle;
    #[cfg(not(any(v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_25
    pub fn IupSpace() -> *mut Ihandle;

    pub fn IupRadio(child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupVbox(child: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupVboxv(children: *mut *mut Ihandle) -> *mut Ihandle;
    pub fn IupZbox(child: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupZboxv(children: *mut *mut Ihandle) -> *mut Ihandle;
    pub fn IupHbox(child: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupHboxv(children: *mut *mut Ihandle) -> *mut Ihandle;

    pub fn IupNormalizer(ih_first: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupNormalizerv(ih_list: *mut *mut Ihandle) -> *mut Ihandle;

    pub fn IupCbox(child: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupCboxv(children: *mut *mut Ihandle) -> *mut Ihandle;
    pub fn IupSbox(child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupSplit(child1: *mut Ihandle, child2: *mut Ihandle) -> *mut Ihandle;
    pub fn IupScrollBox(child: *mut Ihandle) -> *mut Ihandle;
    #[cfg(not(any(v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_22
    pub fn IupFlatScrollBox(child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupGridBox(child: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupGridBoxv(children: *mut *mut Ihandle) -> *mut Ihandle;
    #[cfg(not(any(v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_26
    pub fn IupMultiBox(child: *mut Ihandle, ...) -> *mut Ihandle;
    #[cfg(not(any(v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_26
    pub fn IupMultiBoxv(children: *mut *mut Ihandle) -> *mut Ihandle;
    pub fn IupExpander(child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupDetachBox(child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupBackgroundBox(child: *mut Ihandle) -> *mut Ihandle;

    pub fn IupFrame(child: *mut Ihandle) -> *mut Ihandle;
    #[cfg(not(any(v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_20
    pub fn IupFlatFrame(child: *mut Ihandle) -> *mut Ihandle;

    pub fn IupImage(width: c_int, height: c_int, pixels: *const c_uchar) -> *mut Ihandle;
    pub fn IupImageRGB(width: c_int, height: c_int, pixels: *const c_uchar) -> *mut Ihandle;
    pub fn IupImageRGBA(width: c_int, height: c_int, pixels: *const c_uchar) -> *mut Ihandle;

    pub fn IupItem(title: *const c_char, action: *const c_char) -> *mut Ihandle;
    pub fn IupSubmenu(title: *const c_char, child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupSeparator() -> *mut Ihandle;
    pub fn IupMenu(child: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupMenuv(children: *mut *mut Ihandle) -> *mut Ihandle;

    pub fn IupButton(title: *const c_char, action: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_14, v3_13, v3_12)))] // since v3_15
    pub fn IupFlatButton(title: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_25
    pub fn IupFlatToggle(title: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_25
    pub fn IupDropButton(dropchild: *mut Ihandle) -> *mut Ihandle;
    #[cfg(not(any(v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_25
    pub fn IupFlatLabel(title: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_25
    pub fn IupFlatSeparator() -> *mut Ihandle;
    pub fn IupCanvas(action: *const c_char) -> *mut Ihandle;
    pub fn IupDialog(child: *mut Ihandle) -> *mut Ihandle;
    pub fn IupUser() -> *mut Ihandle;
    #[cfg(not(any(v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_28
    pub fn IupThread() -> *mut Ihandle;
    pub fn IupLabel(title: *const c_char) -> *mut Ihandle;
    pub fn IupList(action: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_27
    pub fn IupFlatList() -> *mut Ihandle;
    pub fn IupText(action: *const c_char) -> *mut Ihandle;
    pub fn IupMultiLine(action: *const c_char) -> *mut Ihandle;
    pub fn IupToggle(title: *const c_char, action: *const c_char) -> *mut Ihandle;
    pub fn IupTimer() -> *mut Ihandle;
    pub fn IupClipboard() -> *mut Ihandle;
    pub fn IupProgressBar() -> *mut Ihandle;
    pub fn IupVal(r#type: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_28
    pub fn IupFlatVal(r#type: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_28, v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_29
    pub fn IupFlatTree() -> *mut Ihandle;
    pub fn IupTabs(child: *mut Ihandle, ...) -> *mut Ihandle;
    pub fn IupTabsv(children: *mut *mut Ihandle) -> *mut Ihandle;
    #[cfg(not(any(v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_21
    pub fn IupFlatTabs(first: *mut Ihandle, ...) -> *mut Ihandle;
    #[cfg(not(any(v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_21
    pub fn IupFlatTabsv(children: *mut *mut Ihandle) -> *mut Ihandle;
    pub fn IupTree() -> *mut Ihandle;
    pub fn IupLink(url: *const c_char, title: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_17
    pub fn IupAnimatedLabel(animation: *mut Ihandle) -> *mut Ihandle;
    #[cfg(not(any(v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_17
    pub fn IupDatePick() -> *mut Ihandle;
    #[cfg(not(any(v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_17
    pub fn IupCalendar() -> *mut Ihandle;
    #[cfg(not(any(v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_24
    pub fn IupColorbar() -> *mut Ihandle;
    #[cfg(not(any(v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_24
    pub fn IupGauge() -> *mut Ihandle;
    #[cfg(not(any(v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_24
    pub fn IupDial(r#type: *const c_char) -> *mut Ihandle;
    #[cfg(not(any(v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_24
    pub fn IupColorBrowser() -> *mut Ihandle;

    ////////
    //                      Utilities                                       
    ////////
    // String compare utility 
    #[cfg(not(any(v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_17
    pub fn IupStringCompare(str1: *const c_char, str2: *const c_char, casesensitive: c_int, lexicographic: c_int) -> c_int;

    // IupImage utilities 
    pub fn IupSaveImageAsText(ih: *mut Ihandle, filename: *const c_char, format: *const c_char, name: *const c_char) -> c_int;
    #[cfg(not(any(v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_28
    pub fn IupImageGetHandle(name: *const c_char) -> *mut Ihandle;

    // IupText and IupScintilla utilities 
    pub fn IupTextConvertLinColToPos(ih: *mut Ihandle, lin: c_int, col: c_int, pos: *mut c_int);
    pub fn IupTextConvertPosToLinCol(ih: *mut Ihandle, pos: c_int, lin: *mut c_int, col: *mut c_int);

    // IupText, IupList, IupTree, IupMatrix and IupScintilla utility 
    pub fn IupConvertXYToPos(ih: *mut Ihandle, x: c_int, y: c_int) -> c_int;

    // IupTree and IupFlatTree utilities (work for both) 
    pub fn IupTreeSetUserId(ih: *mut Ihandle, id: c_int, userid: *mut c_void) -> c_int;
    pub fn IupTreeGetUserId(ih: *mut Ihandle, id: c_int) -> *mut c_void;
    pub fn IupTreeGetId(ih: *mut Ihandle, userid: *mut c_void) -> c_int;
    #[deprecated(since = "3.21", note = "use IupSetAttributeHandleId")]
    pub fn IupTreeSetAttributeHandle(ih: *mut Ihandle, name: *const c_char, id: c_int, ih_named: *mut Ihandle);

    ////////
    //                      Pre-definided dialogs                           
    ////////
    pub fn IupFileDlg() -> *mut Ihandle;
    pub fn IupMessageDlg() -> *mut Ihandle;
    pub fn IupColorDlg() -> *mut Ihandle;
    pub fn IupFontDlg() -> *mut Ihandle;
    pub fn IupProgressDlg() -> *mut Ihandle;

    pub fn IupGetFile(arq: *mut c_char) -> c_int;
    pub fn IupMessage(title: *const c_char, msg: *const c_char);
    pub fn IupMessagef(title: *const c_char, format: *const c_char, ...);
    #[cfg(not(any(v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_22
    pub fn IupMessageError(parent: *mut Ihandle, message: *const c_char);
    #[cfg(not(any(v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_22
    pub fn IupMessageAlarm(parent: *mut Ihandle, title: *const c_char, message: *const c_char, buttons: *const c_char) -> c_int;
    pub fn IupAlarm(title: *const c_char, msg: *const c_char, b1: *const c_char, b2: *const c_char, b3: *const c_char) -> c_int;
    pub fn IupScanf(format: *const c_char, ...) -> c_int;
    pub fn IupListDialog(r#type: c_int, title: *const c_char, size: c_int, list: *mut *const c_char, op: c_int, max_col: c_int, max_lin: c_int, marks: *mut c_int) -> c_int;

    // signature of IupGetText changed from 3.16 -> 3.17
    #[cfg(any(v3_12, v3_13, v3_14, v3_15, v3_16))] // between v3_12 and v3_16
    pub fn IupGetText(title: *const c_char, text: *mut c_char) -> c_int;
    #[cfg(any(warningABI, v3_17, v3_18, v3_19, v3_20, v3_21, v3_22, v3_23, v3_24, v3_25, v3_26, v3_27, v3_28, v3_29, v3_30))] // since v3_17
    pub fn IupGetText(title: *const c_char, text: *mut c_char, maxsize: c_int) -> c_int;

    pub fn IupGetColor(x: c_int, y: c_int, r: *mut c_uchar, g: *mut c_uchar, b: *mut c_uchar) -> c_int;

    pub fn IupGetParam(title: *const c_char, action: Iparamcb, user_data: *mut c_void, format: *const c_char, ...) -> c_int;
    pub fn IupGetParamv(title: *const c_char, action: Iparamcb, user_data: *mut c_void, format: *const c_char, param_count: c_int, param_extra: c_int, param_data: *mut *mut c_void) -> c_int;
    #[cfg(not(any(v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_19
    pub fn IupParam(format: *const c_char) -> *mut Ihandle;
    #[cfg(any(v3_13, v3_14, v3_15, v3_16, v3_17, v3_18))] // between v3_13 and v3_18
    pub fn IupParamf(format: *const c_char) -> *mut Ihandle;

    // signature of IupParamBox changed from 3.18 -> 3.19
    #[cfg(any(v3_13, v3_14, v3_15, v3_16, v3_17, v3_18))] // between v3_13 and v3_18
    pub fn IupParamBox(parent: *mut Ihandle, params: *mut *mut Ihandle, count: c_int) -> *mut Ihandle;
    #[cfg(any(warningABI, v3_19, v3_20, v3_21, v3_22, v3_23, v3_24, v3_25, v3_26, v3_27, v3_28, v3_29, v3_30))] // since v3_19
    pub fn IupParamBox(param: *mut Ihandle, ...) -> *mut Ihandle;

    #[cfg(not(any(v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_19
    pub fn IupParamBoxv(param_array: *mut *mut Ihandle) -> *mut Ihandle;
    pub fn IupLayoutDialog(dialog: *mut Ihandle) -> *mut Ihandle;

    // signature of IupElementPropertiesDialog changed from 3.27 -> 3.28
    #[cfg(any(v3_12, v3_13, v3_14, v3_15, v3_16, v3_17, v3_18, v3_19, v3_20, v3_21, v3_22, v3_23, v3_24, v3_25, v3_26, v3_27))] // between v3_12 and v3_27
    pub fn IupElementPropertiesDialog(                      elem: *mut Ihandle) -> *mut Ihandle;
    #[cfg(any(warningABI, v3_28, v3_29, v3_30))] // since v3_28
    pub fn IupElementPropertiesDialog(parent: *mut Ihandle, elem: *mut Ihandle) -> *mut Ihandle;

    #[cfg(not(any(v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_27
    pub fn IupGlobalsDialog() -> *mut Ihandle;
    #[cfg(not(any(v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_28
    pub fn IupClassInfoDialog(parent: *mut Ihandle) -> *mut Ihandle;
} // extern

////////
//                   Common Flags and Return Values                     
////////

////////
//                   Callback Return Values                             
////////
pub const IUP_IGNORE: c_int     = -1;
pub const IUP_DEFAULT: c_int    = -2;
pub const IUP_CLOSE: c_int      = -3;
pub const IUP_CONTINUE: c_int   = -4;

////////
//           IupPopup and IupShowXY Parameter Values                    
////////
pub const IUP_CENTER: c_int       = 0xFFFF;  // 65535 
pub const IUP_LEFT: c_int         = 0xFFFE;  // 65534 
pub const IUP_RIGHT: c_int        = 0xFFFD;  // 65533 
pub const IUP_MOUSEPOS: c_int     = 0xFFFC;  // 65532 
pub const IUP_CURRENT: c_int      = 0xFFFB;  // 65531 
pub const IUP_CENTERPARENT: c_int = 0xFFFA;  // 65530 
#[cfg(not(any(v3_28, v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_29
pub const IUP_LEFTPARENT: c_int   = 0xFFF9;  // 65529 
#[cfg(not(any(v3_28, v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_29
pub const IUP_RIGHTPARENT: c_int  = 0xFFF8;  // 65528 
pub const IUP_TOP: c_int          = IUP_LEFT;
pub const IUP_BOTTOM: c_int       = IUP_RIGHT;
#[cfg(not(any(v3_28, v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_29
pub const IUP_TOPPARENT: c_int    = IUP_LEFTPARENT;
#[cfg(not(any(v3_28, v3_27, v3_26, v3_25, v3_24, v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_29
pub const IUP_BOTTOMPARENT: c_int = IUP_RIGHTPARENT;

////////
//               SHOW_CB Callback Values                                
////////
pub const IUP_SHOW: c_int     = 0;
pub const IUP_RESTORE: c_int  = 1;
pub const IUP_MINIMIZE: c_int = 2;
pub const IUP_MAXIMIZE: c_int = 3;
pub const IUP_HIDE: c_int     = 4;

////////
//               SCROLL_CB Callback Values                              
////////
pub const IUP_SBUP: c_int      =  0;
pub const IUP_SBDN: c_int      =  1;
pub const IUP_SBPGUP: c_int    =  2;
pub const IUP_SBPGDN: c_int    =  3;
pub const IUP_SBPOSV: c_int    =  4;
pub const IUP_SBDRAGV: c_int   =  5;
pub const IUP_SBLEFT: c_int    =  6;
pub const IUP_SBRIGHT: c_int   =  7;
pub const IUP_SBPGLEFT: c_int  =  8;
pub const IUP_SBPGRIGHT: c_int =  9;
pub const IUP_SBPOSH: c_int    = 10;
pub const IUP_SBDRAGH: c_int   = 11;

////////
//               Mouse Button Values and Functions                      
////////
pub const IUP_BUTTON1: c_int = '1' as c_int;
pub const IUP_BUTTON2: c_int = '2' as c_int;
pub const IUP_BUTTON3: c_int = '3' as c_int;
pub const IUP_BUTTON4: c_int = '4' as c_int;
pub const IUP_BUTTON5: c_int = '5' as c_int;

#[inline(always)]
pub unsafe fn iup_isshift(s: *const c_char) -> bool   { *s.offset(0) == 'S' as c_char }
#[inline(always)]
pub unsafe fn iup_iscontrol(s: *const c_char) -> bool { *s.offset(1) == 'C' as c_char }
#[inline(always)]
pub unsafe fn iup_isbutton1(s: *const c_char) -> bool { *s.offset(2) == '1' as c_char }
#[inline(always)]
pub unsafe fn iup_isbutton2(s: *const c_char) -> bool { *s.offset(3) == '2' as c_char }
#[inline(always)]
pub unsafe fn iup_isbutton3(s: *const c_char) -> bool { *s.offset(4) == '3' as c_char }
#[inline(always)]
pub unsafe fn iup_isdouble(s: *const c_char) -> bool  { *s.offset(5) == 'D' as c_char }
#[inline(always)]
pub unsafe fn iup_isalt(s: *const c_char) -> bool     { *s.offset(6) == 'A' as c_char }
#[inline(always)]
pub unsafe fn iup_issys(s: *const c_char) -> bool     { *s.offset(7) == 'Y' as c_char }
#[inline(always)]
pub unsafe fn iup_isbutton4(s: *const c_char) -> bool { *s.offset(8) == '4' as c_char }
#[inline(always)]
pub unsafe fn iup_isbutton5(s: *const c_char) -> bool { *s.offset(9) == '5' as c_char }

////////
//                      Pre-Defined Masks                               
////////
pub const IUP_MASK_FLOAT: &'static str       = "[+/-]?(/d+/.?/d*|/./d+)";
pub const IUP_MASK_UFLOAT: &'static str      =       "(/d+/.?/d*|/./d+)";
pub const IUP_MASK_EFLOAT: &'static str      = "[+/-]?(/d+/.?/d*|/./d+)([eE][+/-]?/d+)?";
#[cfg(not(any(v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_23
pub const IUP_MASK_UEFLOAT: &'static str     =       "(/d+/.?/d*|/./d+)([eE][+/-]?/d+)?";
#[cfg(not(any(v3_12)))] // since v3_13
pub const IUP_MASK_FLOATCOMMA: &'static str  = "[+/-]?(/d+/,?/d*|/,/d+)";
#[cfg(not(any(v3_12)))] // since v3_13
pub const IUP_MASK_UFLOATCOMMA: &'static str =       "(/d+/,?/d*|/,/d+)";
pub const IUP_MASK_INT: &'static str         =  "[+/-]?/d+";
pub const IUP_MASK_UINT: &'static str        =        "/d+";

////////
//                   IupGetParam Callback situations                    
////////
pub const IUP_GETPARAM_BUTTON1: c_int = -1;
pub const IUP_GETPARAM_INIT: c_int    = -2;
pub const IUP_GETPARAM_BUTTON2: c_int = -3;
pub const IUP_GETPARAM_BUTTON3: c_int = -4;
#[cfg(not(any(v3_12)))] // since v3_13
pub const IUP_GETPARAM_CLOSE: c_int   = -5;
#[cfg(not(any(v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_21
pub const IUP_GETPARAM_MAP: c_int     = -6;
pub const IUP_GETPARAM_OK: c_int      = IUP_GETPARAM_BUTTON1;
pub const IUP_GETPARAM_CANCEL: c_int  = IUP_GETPARAM_BUTTON2;
pub const IUP_GETPARAM_HELP: c_int    = IUP_GETPARAM_BUTTON3;

////////
//                   Used by IupColorbar                                
////////
#[cfg(not(any(v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_24
pub const IUP_PRIMARY: c_int   = -1;
#[cfg(not(any(v3_23, v3_22, v3_21, v3_20, v3_19, v3_18, v3_17, v3_16, v3_15, v3_14, v3_13, v3_12)))] // since v3_24
pub const IUP_SECONDARY: c_int = -2;

////////
//                   Record Input Modes                                 
////////
pub const IUP_RECBINARY: c_int = 0;
pub const IUP_RECTEXT: c_int   = 1;
*/
