extern crate libc;
#[cfg(all(unix, not(target_os = "macos")))]
use libc::gid_t;
use std::ffi::CString;

pub fn is_write_allowed(folder_path: &str) -> std::result::Result<bool, &'static str> {
    let c_string = CString::new(folder_path).unwrap();
    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        let res = libc::stat(c_string.as_ptr(), &mut stat);

        if res != 0 {
            return Err("Unable to stat() directory");
        }

        let mode = stat.st_mode;
        if stat.st_uid == libc::geteuid() {
            return Ok(mode & libc::S_IWUSR != 0);
        }
        if stat.st_gid == libc::getgid() {
            return Ok(mode & libc::S_IWGRP != 0);
        }

        let num_groups = libc::getgroups(0, ::std::ptr::null_mut());
        if num_groups == -1 {
            return Err("Unable to get suplementary groups for the current process");
        }
        #[cfg(all(unix, target_os = "macos"))]
        let mut groups: Vec<u32> = vec![0; 1024];
        #[cfg(all(unix, not(target_os = "macos")))]
        let mut groups: Vec<gid_t> = vec![0; 1024];
        let res = libc::getgroups(num_groups, groups.as_mut_ptr());
        if res == -1 {
            return Err("Unable to get suplementary groups for the current process");
        }

        for i in 0..num_groups {
            if groups[i as usize] == stat.st_gid {
                return Ok(mode & libc::S_IWGRP != 0);
            }
        }

        Ok(mode & libc::S_IWOTH != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_test() {
        assert_eq!(is_write_allowed("/etc"), Ok(false));
        assert_eq!(
            is_write_allowed("/i_dont_exist"),
            Err("Unable to stat() directory")
        );
    }
}
