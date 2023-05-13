mod libsshgen;

#[allow(dead_code)]

use std::marker::PhantomData;
use std::mem::transmute;
use std::ptr::null_mut;

pub use libsshgen::ssh_options_e as ssh_options;
pub use libsshgen::ssh_keytypes_e as ssh_keytypes;
pub use libsshgen::ssh_auth_e as ssh_auth;

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
pub struct Authenticated;
impl SessionStatus for Setup {}
impl SessionStatus for Connected {}
impl SessionStatus for Authenticated {}

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

impl<T: SessionStatus> Drop for SSHSession<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { libsshgen::ssh_free(self.ptr) };
        }
    }
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
    ///
    /// # Return
    /// Returns a Result<SSHSession<Connected>, i32> object
    pub fn connect(mut self) -> Result<SSHSession<Connected>, i32> {
        let res = unsafe { libsshgen::ssh_connect(self.ptr) };

        match res {
            0 => {
                // ensure we don't erase this ptr since we're transforming
                // rather than creating
                let ptr = std::mem::replace(&mut self.ptr, null_mut());
                Ok(SSHSession {
                    ptr,
                    _marker: PhantomData,
                })
            }
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
    pub fn disconnect(mut self) -> SSHSession<Setup> {
        unsafe { libsshgen::ssh_disconnect(self.ptr) };
        // we don't want to delete the ptr, so we replace it with NULL in the original
        let ptr = std::mem::replace(&mut self.ptr, null_mut());
        SSHSession {
            ptr,
            _marker: PhantomData,
        }
    }
    /// Disconnects immediately from the server by closing the socket
    ///
    /// Note that after disconnecting the session can be re-used to connect again.
    pub fn silent_disconnect(mut self) -> SSHSession<Setup> {
        unsafe { libsshgen::ssh_silent_disconnect(self.ptr) };
        // we don't want to delete the ptr, so we replace it with NULL in the original
        let ptr = std::mem::replace(&mut self.ptr, null_mut());
        SSHSession {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Tries to authenticate with a provided public key.
    ///
    /// # Arguments
    ///
    /// * key : the key object to authenticate with
    pub fn userauth_try_publickey(&self, key: &PubKey) -> ssh_auth {
        unsafe { transmute(libsshgen::ssh_userauth_try_publickey(self.ptr, std::ptr::null(), key.ptr)) }
    }
}

// public key functions

/// a PubKey is an opaque wrapper around a libssh ssh_key structure
/// specifically limited to public key-related functions
///
/// Note that this is marked Sync as these are thread safe in the base lib
#[derive(Debug)]
pub struct PubKey {
    ptr: libsshgen::ssh_key,
}
unsafe impl Sync for PubKey {}
unsafe impl Send for PubKey {}

impl Drop for PubKey {
    fn drop(&mut self) {
        unsafe { libsshgen::ssh_key_free(self.ptr); }
    }
}

#[derive(Debug)]
pub enum Error {
    Alloc,
    UTF8,
    Parse,
}

impl PubKey {
    /// Imports a public key of the given type from a base64 string
    ///
    /// # Arguments
    /// * k   : the base64 encoded public key
    /// * typ : the type of key to interprete the bytes as
    ///
    /// # Returns
    /// Either a new PubKey struct or an Error value
    pub fn from_base64(k: &str, typ: ssh_keytypes) -> Result<PubKey, Error> {
        let mut ptr: libsshgen::ssh_key = unsafe { libsshgen::ssh_key_new() };
        if ptr.is_null() {
            return Err(Error::Alloc);
        }
        
        std::ffi::CString::new(k.clone())
            .map_err(|_| Error::UTF8)
            .and_then(|c| Ok(c.into_raw()))
            .and_then(|c| {
                let res = unsafe { libsshgen::ssh_pki_import_pubkey_base64(c, typ, &mut ptr) };
                if res == 0 {
                    Ok(PubKey { ptr, })
                }
                else {
                    Err(Error::Parse)
                }
            })
    }

    /// Imports a public key from a given file
    ///
    /// # Arguments
    /// * f : the file containing a public key to import
    pub fn from_file(f: &std::path::PathBuf) -> Result<PubKey, Error> {
        let mut ptr: libsshgen::ssh_key = unsafe { libsshgen::ssh_key_new() };
        if ptr.is_null() {
            return Err(Error::Alloc);
        }

        std::ffi::CString::new(f.to_str().expect("Invalid file path"))
            .map_err(|_| Error::UTF8)
            .and_then(|c| Ok(c.into_raw()))
            .and_then(|f| {
                let res = unsafe { libsshgen::ssh_pki_import_pubkey_file(f, &mut ptr) };
                if res == 0 {
                    Ok(PubKey { ptr, })
                }
                else {
                    Err(Error::Parse)
                }
            })
    }
}
