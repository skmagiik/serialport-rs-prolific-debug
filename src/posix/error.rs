use std::error::Error;
use std::ffi::CStr;
use std::io;
use std::str;

use libc::{self, c_int, c_char, size_t};
#[cfg(target_os = "linux")]
use libudev;
use nix;

pub fn last_os_error() -> ::Error {
    from_raw_os_error(nix::errno::errno())
}

pub fn from_raw_os_error(errno: i32) -> ::Error {
    use libc::{EBUSY, EISDIR, ELOOP, ENOTDIR, ENOENT, ENODEV, ENXIO, EACCES, EINVAL, ENAMETOOLONG,
               EINTR, EWOULDBLOCK};

    let kind = match errno {
        EBUSY | EISDIR | ELOOP | ENOTDIR | ENOENT | ENODEV | ENXIO | EACCES => {
            ::ErrorKind::NoDevice
        }
        EINVAL | ENAMETOOLONG => ::ErrorKind::InvalidInput,

        EINTR => ::ErrorKind::Io(io::ErrorKind::Interrupted),
        EWOULDBLOCK => ::ErrorKind::Io(io::ErrorKind::WouldBlock),
        _ => ::ErrorKind::Io(io::ErrorKind::Other),
    };

    ::Error::new(kind, error_string(errno))
}

pub fn from_io_error(io_error: io::Error) -> ::Error {
    match io_error.raw_os_error() {
        Some(errno) => from_raw_os_error(errno),
        None => {
            let description = io_error.description().to_string();

            ::Error::new(::ErrorKind::Io(io_error.kind()), description)
        }
    }
}

#[cfg(target_os = "linux")]
impl From<libudev::Error> for ::Error {
    fn from(e: libudev::Error) -> ::Error {
        let description = e.description().to_string();
        match e.kind() {
            libudev::ErrorKind::NoMem => ::Error::new(::ErrorKind::Unknown, description),
            libudev::ErrorKind::InvalidInput => {
                ::Error::new(::ErrorKind::InvalidInput, description)
            }
            libudev::ErrorKind::Io(a) => ::Error::new(::ErrorKind::Io(a), description),
        }
    }
}

// the rest of this module is borrowed from libstd
const TMPBUF_SZ: usize = 128;

pub fn error_string(errno: i32) -> String {

    let mut buf = [0 as c_char; TMPBUF_SZ];

    let p = buf.as_mut_ptr();
    unsafe {
        if libc::strerror_r(errno as c_int, p, buf.len() as size_t) < 0 {
            panic!("strerror_r failure");
        }

        let p = p as *const _;
        str::from_utf8(CStr::from_ptr(p).to_bytes()).unwrap().to_string()
    }
}
