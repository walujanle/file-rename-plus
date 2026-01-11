// Security: privilege detection and file access validation

use std::fs::{self, OpenOptions};
use std::path::Path;

// Checks if file can be modified (considers permissions and admin status)
pub fn can_modify_file(path: &Path) -> bool {
    if !path.exists() {
        return path.parent().map(can_write_to_directory).unwrap_or(false);
    }

    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.permissions().readonly() && !is_running_as_admin() {
                return false;
            }
            can_open_for_write(path)
        }
        Err(_) => false,
    }
}

// Tests write access by creating temp file
fn can_write_to_directory(dir: &Path) -> bool {
    if !dir.exists() || !dir.is_dir() {
        return false;
    }
    let test_file = dir.join(format!(".write_test_{}", std::process::id()));
    match fs::File::create(&test_file) {
        Ok(file) => {
            drop(file);
            let _ = fs::remove_file(&test_file);
            true
        }
        Err(_) => false,
    }
}

// Tests if file can be opened for writing
fn can_open_for_write(path: &Path) -> bool {
    OpenOptions::new().write(true).open(path).is_ok()
}

// Checks if running as admin (Windows)
#[cfg(target_os = "windows")]
fn is_running_as_admin() -> bool {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Security::{
        AllocateAndInitializeSid, CheckTokenMembership, FreeSid, PSID, SID_IDENTIFIER_AUTHORITY,
    };

    const SECURITY_NT_AUTHORITY: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    const SECURITY_BUILTIN_DOMAIN_RID: u32 = 32;
    const DOMAIN_ALIAS_RID_ADMINS: u32 = 544;

    let mut admin_group: PSID = PSID::default();

    // Allocate SID - unsafe block minimized
    let alloc_result = unsafe {
        AllocateAndInitializeSid(
            &SECURITY_NT_AUTHORITY,
            2,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut admin_group,
        )
    };

    if alloc_result.is_err() {
        return false;
    }

    // Check membership - unsafe block minimized
    let mut is_member: BOOL = BOOL(0);
    let check_result = unsafe { CheckTokenMembership(None, admin_group, &mut is_member) };

    // Always free the SID
    unsafe {
        let _ = FreeSid(admin_group);
    }

    check_result.is_ok() && is_member.as_bool()
}

// Checks if running as root (Unix)
#[cfg(target_family = "unix")]
fn is_running_as_admin() -> bool {
    // SAFETY: getuid() is always safe to call
    unsafe { libc::getuid() == 0 }
}

// Fallback for other platforms
#[cfg(not(any(target_os = "windows", target_family = "unix")))]
fn is_running_as_admin() -> bool {
    false
}
