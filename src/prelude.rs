// Copyright © 2020 Mark Summerfield. All rights reserved.
// Licensed under the Apache License, Version 2.0.

use crate::xerror::XResult;
use std::ffi::{CStr, CString};

pub(crate) fn c_to_string(p: *const i8) -> XResult<String> {
    let c: &CStr = unsafe { CStr::from_ptr(p) };
    let s: &str = c.to_str()?;
    Ok(s.to_owned())
}

pub(crate) fn c_from_str(s: &str) -> *const i8 {
    CString::new(s).unwrap().into_raw()
}

#[repr(C)] pub struct Ihandle { _private: [u8; 0] }
pub type Icallback = extern fn(ih: *mut Ihandle) -> i32;

pub const ERROR: i32 = 1;
pub const NOERROR: i32 = 0;
pub const OPENED: i32 = -1;
pub const INVALID: i32 = -1;
pub const INVALID_ID: i32 = -10;

pub const IGNORE: i32 = -1;
pub const DEFAULT: i32 = -2;
pub const CLOSE: i32 = -3;
pub const CONTINUE: i32 = -4;

pub const ACTION: &str = "ACTION";
pub const ACTION_CB: &str = "ACTION_CB";
pub const BRINGFRONT: &str = "BRINGFRONT";
pub const ICON: &str = "ICON";
pub const NAME: &str = "NAME";
pub const RUN: &str = "RUN";
pub const SYSTEM: &str = "SYSTEM";
pub const SYSTEMVERSION: &str = "SYSTEMVERSION";
pub const TIME: &str = "TIME";
pub const TITLE: &str = "TITLE";

pub const YES: &str = "YES";
pub const NO: &str = "NO";

pub const CENTER: i32 = 0xFFFF;
pub const LEFT: i32 = 0xFFFE;
pub const RIGHT: i32 = 0xFFFD;
pub const MOUSEPOS: i32 = 0xFFFC;
pub const CURRENT: i32 = 0xFFFB;
pub const CENTERPARENT: i32 = 0xFFFA;
pub const LEFTPARENT: i32 = 0xFFF9;
pub const RIGHTPARENT: i32 = 0xFFF8;
pub const TOP: i32 = LEFT;
pub const BOTTOM: i32 = RIGHT;
pub const TOPPARENT: i32 = LEFTPARENT;
pub const BOTTOMPARENT: i32 = RIGHTPARENT;

pub(crate) const UTF8MODE: &str = "UTF8MODE";
