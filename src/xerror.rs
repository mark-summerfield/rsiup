// Copyright © 2018-19 Mark Summerfield. All rights reserved.
// Licensed under the Apache License, Version 2.0.

use std::error::Error;
use std::io;
use std::fmt;

pub type XResult<T> = Result<T, Box<XError>>;

#[inline]
pub fn xerror<T>(message: impl Into<String>) -> XResult<T> {
    Err(Box::new(XError::new(message)))
}

#[macro_export]
macro_rules! xerr {
 ($msg:expr $(,)?) => (return xerror($msg));
 ($fmt:expr $(, $y:expr)+ $(,)?) => (return xerror(format!($fmt, $($y),*)));
}

#[derive(Debug)]
pub enum XError {
    Dll(libloading::Error),
    Error(String),
    Io(io::Error),
    Utf8Encoding(::std::string::FromUtf8Error),
    Utf8Decoding(::std::str::Utf8Error),
}

impl Error for XError {}

impl XError {
    #[inline]
    pub fn new(message: impl Into<String>) -> XError {
        XError::Error(message.into())
    }
}

impl fmt::Display for XError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            XError::Dll(ref err) => write!(out, "{}", err),
            XError::Error(ref err) => write!(out, "{}", err),
            XError::Io(ref err) => write!(out, "File error: {}", err),
            XError::Utf8Encoding(ref err) => {
                write!(out, "Encoding error: {}", err)
            }
            XError::Utf8Decoding(ref err) => {
                write!(out, "Decoding error: {}", err)
            }
        }
    }
}

impl From<libloading::Error> for Box<XError> {
    #[inline]
    fn from(err: libloading::Error) -> Box<XError> {
        Box::new(XError::Dll(err))
    }
}

impl From<io::Error> for Box<XError> {
    #[inline]
    fn from(err: io::Error) -> Box<XError> {
        Box::new(XError::Io(err))
    }
}

impl From<::std::string::FromUtf8Error> for Box<XError> {
    #[inline]
    fn from(err: ::std::string::FromUtf8Error) -> Box<XError> {
        Box::new(XError::Utf8Encoding(err))
    }
}

impl From<::std::str::Utf8Error> for Box<XError> {
    #[inline]
    fn from(err: ::std::str::Utf8Error) -> Box<XError> {
        Box::new(XError::Utf8Decoding(err))
    }
}
