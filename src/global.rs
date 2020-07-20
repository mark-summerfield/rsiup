// Copyright © 2020 Mark Summerfield. All rights reserved.

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
pub const NAME: &str = "NAME";
pub const RUN: &str = "RUN";
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
