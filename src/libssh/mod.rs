mod libsshgen;

use std::marker::PhantomData;

pub use libsshgen::ssh_options_e as ssh_options;
use libsshgen::ssh_session as raw_ssh_session;

pub use libsshgen::SSH_AGAIN;
pub use libsshgen::SSH_ERROR;
pub use libsshgen::SSH_OK;

// wrappings for what I need

// type errors aren't fun :)
mod private {
    pub trait SessionStatus {}
}
use private::SessionStatus;
pub struct Setup;
pub struct Connected;
impl SessionStatus for Setup {}
impl SessionStatus for Connected {}

/// An SSHSession is an opaque object representing an SSH session.
/// A session is a single connection to the server and must
/// be properly authenticated in order to be useful (for most purposes).
///
/// After authentication, the session can create channels to send/receive data over.
#[derive(Debug)]
pub struct SSHSession<T: SessionStatus> {
    ptr: raw_ssh_session,
    _marker: PhantomData<T>,
}

impl SSHSession<Setup> {
    /// Returns a new SSHSession in Setup.
    /// Use the `options_set_*` functions to configure it
    /// before calling `connect()`.
    pub fn new() -> SSHSession<Setup> {
        let session = unsafe { libsshgen::ssh_new() };

        SSHSession {
            ptr: session,
            _marker: PhantomData,
        }
    }

    /// Handles ssh_options configured via raw c_strings
    fn options_set_str(&self, opt: ssh_options, str: &str) {
        let c_str = std::ffi::CString::new(str.clone()).expect("Host contained NULL");
        let err = unsafe {
            libsshgen::ssh_options_set(self.ptr, opt, c_str.as_ptr() as *const std::ffi::c_void)
        };
        if err < 0 {
            let msg = unsafe {
                let raw = libsshgen::ssh_get_error(self.ptr as *mut std::os::raw::c_void);
                std::ffi::CStr::from_ptr(raw)
            }
            .to_str()
            .expect("LibSSH has a non-UTF error string");
            panic!("{}", msg);
        }
    }

    /// Configures the host to connect to when running `connect()`
    pub fn options_set_host(&self, host: &str) {
        self.options_set_str(ssh_options::SSH_OPTIONS_HOST, host)
    }
    /// Configures the port to use when running `connect()`, passed as a string
    pub fn options_set_port_str(&self, port_str: &str) {
        self.options_set_str(ssh_options::SSH_OPTIONS_PORT_STR, port_str)
    }
    /// Configures the username to use when running `connect()`
    pub fn options_set_user(&self, user: &str) {
        self.options_set_str(ssh_options::SSH_OPTIONS_USER, user)
    }

    /// Connects to the configured remote host
    ///
    /// The host and various options are configured using the `options_set_*`
    /// collection of functions, which modify the SSHSession object internally.
    pub fn connect(self) -> Result<SSHSession<Connected>, i32> {
        let res = unsafe { libsshgen::ssh_connect(self.ptr) };

        match res {
            0 => Ok(SSHSession {
                ptr: self.ptr,
                _marker: PhantomData,
            }),
            _ => Err(res),
        }
    }
}

impl SSHSession<Connected> {
    
     /// Returns the server's Banner.
     ///
     /// The banner is set by the server and includes the ssh server's version information.
     /// An example of such a banner is:
     /// ```
     /// SSH-2.0-OpenSSH_8.4p1 Debian-5+deb11u1
     /// ```
    pub fn get_server_banner(&self) -> Result<&str, ()> {
        let raw = unsafe { libsshgen::ssh_get_serverbanner(self.ptr) };
        if raw.is_null() {
            return Err(());
        }

        let cs = unsafe { std::ffi::CStr::from_ptr(raw) };
        Ok(cs.to_str().expect("Non-utf8 bytes in server banner"))
    }

    /// Disconnects gracefully from the remote server
    ///
    /// Note that after disconnecting the session can be re-used to connect again.
    pub fn disconnect(self) -> SSHSession<Setup> {
        unsafe { libsshgen::ssh_disconnect(self.ptr) };
        SSHSession {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
    /// Disconnects immediately from the server by closing the socket
    ///
    /// Note that after disconnecting the session can be re-used to connect again.
    pub fn silent_disconnect(self) -> SSHSession<Setup> {
        unsafe { libsshgen::ssh_silent_disconnect(self.ptr) };
        SSHSession {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}
