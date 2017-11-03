
extern crate libc;

mod libvmi_c;

use std::ffi::{CString, CStr};
use std::mem::uninitialized;
use std::ptr::null;

// We sometimes need to free pointers returned by libvmi
use libc::free;

// Export all libvmi symbols, but provide a nice convenient wrapper too.
pub use libvmi_c::*;

/// A handle on a VM for LibVMI
pub struct VmiInstance {
    /// The libvmi handle type
    vmi: vmi_instance_t,

    /// Is this VM paused or not
    paused: bool,
}

impl VmiInstance {
    pub fn new(name: &str) -> Result<VmiInstance, vmi_init_error_t> {
        unsafe {
            let mut vmi: vmi_instance_t = uninitialized();
            let mut error: vmi_init_error_t = uninitialized();

            // Attempt to initialize a VMI instance handle
            let result = vmi_init_complete(
                &mut vmi as *mut _,
                CString::new(name).unwrap().as_ptr() as *mut _,
                VMI_INIT_DOMAINNAME as u64,
                null::<u64>() as *mut _,
                vmi_config_VMI_CONFIG_GLOBAL_FILE_ENTRY,
                null::<u64>() as *mut _,
                &mut error as *mut _,
            );

            // On failure extract the error type
            if result == status_VMI_FAILURE {
                return Err(error);
            }

            // Otherwise, return the handle
            Ok(VmiInstance { vmi, paused: false })
        }
    }

    pub fn vmi_get_offset(&mut self, offset_name: &str) -> Result<addr_t, ()> {
        unsafe {
            let mut offset: addr_t = 0u64;

            if vmi_get_offset(
                self.vmi,
                CString::new(offset_name).unwrap().as_ptr() as *mut _,
                &mut offset as *mut _,
            ) == status_VMI_FAILURE
            {
                Err(())
            } else {
                Ok(offset)
            }
        }
    }

    pub fn vmi_pause_vm(&mut self) -> Result<(), ()> {
        unsafe {
            if vmi_pause_vm(self.vmi) == status_VMI_FAILURE {
                Err(())
            } else {
                self.paused = true;
                Ok(())
            }
        }
    }

    pub fn vmi_resume_vm(&mut self) -> Result<(), ()> {
        unsafe {
            if vmi_resume_vm(self.vmi) == status_VMI_FAILURE {
                Err(())
            } else {
                self.paused = false;
                Ok(())
            }
        }
    }

    pub fn vmi_translate_ksym2v(&mut self, name: &str) -> Result<addr_t, ()> {
        unsafe {
            let mut addr: addr_t = uninitialized();

            if vmi_translate_ksym2v(
                self.vmi,
                CString::new(name).unwrap().as_ptr() as *mut _,
                &mut addr as *mut _,
            ) == status_VMI_FAILURE
            {
                Err(())
            } else {
                Ok(addr)
            }
        }
    }

    pub fn vmi_read_addr_va(&mut self, vaddr: addr_t, pid: i32) -> Result<addr_t, ()> {
        unsafe {
            let mut addr: addr_t = uninitialized();

            if vmi_read_addr_va(self.vmi, vaddr, pid as _, &mut addr as *mut _) ==
                status_VMI_FAILURE
            {
                Err(())
            } else {
                Ok(addr)
            }
        }
    }

    pub fn vmi_read_32_va(&mut self, vaddr: addr_t, pid: i32) -> Result<u32, ()> {
        unsafe {
            let mut val: u32 = 0;

            if vmi_read_32_va(self.vmi, vaddr, pid as _, &mut val as *mut _) == status_VMI_FAILURE {
                Err(())
            } else {
                Ok(val)
            }
        }
    }

    pub fn vmi_read_str_va(&mut self, vaddr: addr_t, pid: i32) -> Result<String, ()> {
        unsafe {
            let s = vmi_read_str_va(self.vmi, vaddr, pid as _);

            if s == (null::<i8>() as *mut _) {
                Err(())
            } else {
                // Allocate a normal rust string
                let c_str = CStr::from_ptr(s).to_string_lossy().into_owned();

                // Free the one allocated by libvmi
                free(s as *mut _);

                Ok(c_str)
            }
        }
    }
}

impl Drop for VmiInstance {
    fn drop(&mut self) {
        // Unpause the VM if it is paused
        if let Err(()) = self.vmi_resume_vm() {
            println!("Unable to resume VM before dropping handle");
        }

        // Destroy the handle
        unsafe {
            if vmi_destroy(self.vmi) == status_VMI_FAILURE {
                println!("Unable to destroy handle before dropping");
            }
        }
    }
}
