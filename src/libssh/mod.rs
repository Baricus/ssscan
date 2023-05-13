mod libsshgen;

use std::marker::PhantomData;

use libsshgen::ssh_session as raw_ssh_session;
pub use libsshgen::ssh_options_e as ssh_options;

pub use libsshgen::SSH_OK;
pub use libsshgen::SSH_AGAIN;
pub use libsshgen::SSH_ERROR;

// wrappings for what I need

// type errors aren't fun :)
mod private {
pub trait SessionStatus {}
}
use private::SessionStatus;
pub struct Setup;
pub struct Connected;
pub struct Disconnected;
impl SessionStatus for Setup {}
impl SessionStatus for Connected {}
impl SessionStatus for Disconnected {}

#[derive(Debug)]
pub struct SSHSession<T: SessionStatus> {
    ptr :raw_ssh_session,
    _marker: PhantomData<T>
}


impl SSHSession<Setup> {
    pub fn new() -> SSHSession<Setup> {
        let session = unsafe {
            libsshgen::ssh_new()
        };

        SSHSession { ptr: session, _marker: PhantomData, }
    }

    fn options_set_str(&self, opt: ssh_options, str: &str) {
        let c_str = std::ffi::CString::new(str.clone()).expect("Host contained NULL");   
        let err = unsafe {
            libsshgen::ssh_options_set(self.ptr, opt, c_str.as_ptr() as *const std::ffi::c_void)
        };
        if err < 0 {
            let msg = unsafe { 
                let raw = libsshgen::ssh_get_error(self.ptr as *mut std::os::raw::c_void);
                std::ffi::CStr::from_ptr(raw)
            }.to_str().expect("LibSSH has a non-UTF error string");
            panic!("{}", msg);
        }

    }

    pub fn options_set_host(&self, host: &str) {
        self.options_set_str(ssh_options::SSH_OPTIONS_HOST, host)
    }
    pub fn options_set_port_str(&self, port_str: &str) {
        self.options_set_str(ssh_options::SSH_OPTIONS_PORT_STR, port_str)
    }
    pub fn options_set_user(&self, user: &str) {
        self.options_set_str(ssh_options::SSH_OPTIONS_USER, user)
    }

    pub fn connect(self) -> Result<SSHSession<Connected>, i32> {
        let res = unsafe {libsshgen::ssh_connect(self.ptr)};

        match res {
            0 => Ok(SSHSession { ptr: self.ptr, _marker: PhantomData, }),
            _ => Err(res)
        }
    }
}

impl SSHSession<Connected> {
    pub fn get_server_banner(&self) -> Result<&str, ()> {
        let raw = unsafe { libsshgen::ssh_get_serverbanner(self.ptr) };
        if raw.is_null() {
            return Err(());
        }

        let cs = unsafe { std::ffi::CStr::from_ptr(raw) };
        Ok(cs.to_str().expect("Non-utf8 bytes in server banner"))
    }
}
