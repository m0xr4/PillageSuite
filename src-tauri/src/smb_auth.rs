use std::os::windows::ffi::OsStrExt;

#[link(name = "kernel32")]
extern "system" {
    fn CloseHandle(hObject: isize) -> i32;
}

#[link(name = "advapi32")]
extern "system" {
    fn LogonUserW(
        lpszUsername: *const u16,
        lpszDomain: *const u16,
        lpszPassword: *const u16,
        dwLogonType: u32,
        dwLogonProvider: u32,
        phToken: *mut isize,
    ) -> i32;
    fn ImpersonateLoggedOnUser(hToken: isize) -> i32;
    fn RevertToSelf() -> i32;
}

const LOGON32_LOGON_NEW_CREDENTIALS: u32 = 9;
const LOGON32_PROVIDER_WINNT50: u32 = 3;

/// RAII guard that reverts thread impersonation and closes the token handle on drop.
pub struct ImpersonationGuard {
    token: isize,
    active: bool,
}

impl Drop for ImpersonationGuard {
    fn drop(&mut self) {
        if self.active {
            unsafe { RevertToSelf(); }
        }
        if self.token != 0 {
            unsafe { CloseHandle(self.token); }
        }
    }
}

fn string_to_wide(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Attempt to impersonate a user via LogonUserW with LOGON32_LOGON_NEW_CREDENTIALS.
/// This clones the current token but replaces network credentials (same as `runas /netonly`).
/// No special privileges required.
pub fn start_impersonation(username: &str, password: &str, domain: &str) -> Result<ImpersonationGuard, String> {
    let mut token: isize = 0;
    let username_wide = string_to_wide(username);
    let password_wide = string_to_wide(password);
    let domain_wide = string_to_wide(domain);
    let domain_ptr = if domain.is_empty() {
        std::ptr::null()
    } else {
        domain_wide.as_ptr()
    };

    let success = unsafe {
        LogonUserW(
            username_wide.as_ptr(),
            domain_ptr,
            password_wide.as_ptr(),
            LOGON32_LOGON_NEW_CREDENTIALS,
            LOGON32_PROVIDER_WINNT50,
            &mut token,
        )
    };

    if success == 0 {
        let err = std::io::Error::last_os_error();
        return Err(format!(
            "LogonUserW failed for '{}{}{}': {} (code {})",
            if domain.is_empty() { "" } else { domain },
            if domain.is_empty() { "" } else { "\\" },
            username,
            err,
            err.raw_os_error().unwrap_or(0)
        ));
    }

    let imp_result = unsafe { ImpersonateLoggedOnUser(token) };
    if imp_result == 0 {
        let err = std::io::Error::last_os_error();
        unsafe { CloseHandle(token); }
        return Err(format!("ImpersonateLoggedOnUser failed: {}", err));
    }

    Ok(ImpersonationGuard { token, active: true })
}
