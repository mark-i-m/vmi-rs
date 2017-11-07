
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

    pub fn vmi_get_offset(&mut self, offset_name: &str) -> Result<addr_t, String> {
        unsafe {
            let mut offset: addr_t = 0u64;

            if vmi_get_offset(
                self.vmi,
                CString::new(offset_name).unwrap().as_ptr() as *mut _,
                &mut offset as *mut _,
            ) == status_VMI_FAILURE
            {
                Err(format!(
                    "Unable to get offset \"{}\" from config",
                    offset_name
                ))
            } else {
                Ok(offset)
            }
        }
    }

    pub fn vmi_pause_vm(&mut self) -> Result<(), String> {
        unsafe {
            if vmi_pause_vm(self.vmi) == status_VMI_FAILURE {
                Err("Unable to pause vm".into())
            } else {
                self.paused = true;
                Ok(())
            }
        }
    }

    pub fn vmi_resume_vm(&mut self) -> Result<(), String> {
        unsafe {
            if vmi_resume_vm(self.vmi) == status_VMI_FAILURE {
                Err("Unable to resume vm".into())
            } else {
                self.paused = false;
                Ok(())
            }
        }
    }

    pub fn vmi_translate_ksym2v(&mut self, name: &str) -> Result<addr_t, String> {
        unsafe {
            let mut addr: addr_t = uninitialized();

            if vmi_translate_ksym2v(
                self.vmi,
                CString::new(name).unwrap().as_ptr() as *mut _,
                &mut addr as *mut _,
            ) == status_VMI_FAILURE
            {
                Err(format!(
                    "Unable to translate kernel symbol \"{}\" to va",
                    name
                ))
            } else {
                Ok(addr)
            }
        }
    }

    pub fn vmi_read_addr_va(&mut self, vaddr: addr_t, pid: i32) -> Result<addr_t, String> {
        unsafe {
            let mut addr: addr_t = uninitialized();

            if vmi_read_addr_va(self.vmi, vaddr, pid as _, &mut addr as *mut _) ==
                status_VMI_FAILURE
            {
                Err(format!(
                    "Unable to read addr from address 0x{:X} for PID {}",
                    vaddr,
                    pid
                ))
            } else {
                Ok(addr)
            }
        }
    }

    pub fn vmi_read_32_va(&mut self, vaddr: addr_t, pid: i32) -> Result<u32, String> {
        unsafe {
            let mut val: u32 = 0;

            if vmi_read_32_va(self.vmi, vaddr, pid as _, &mut val as *mut _) == status_VMI_FAILURE {
                Err(format!(
                    "Unable to read u32 from address 0x{:X} for PID {}",
                    vaddr,
                    pid
                ))
            } else {
                Ok(val)
            }
        }
    }

    pub fn vmi_read_64_va(&mut self, vaddr: addr_t, pid: i32) -> Result<u64, String> {
        unsafe {
            let mut val: u64 = 0;

            if vmi_read_64_va(self.vmi, vaddr, pid as _, &mut val as *mut _) == status_VMI_FAILURE {
                Err(format!(
                    "Unable to read u64 from address 0x{:X} for PID {}",
                    vaddr,
                    pid
                ))
            } else {
                Ok(val)
            }
        }
    }

    pub fn vmi_read_str_va(&mut self, vaddr: addr_t, pid: i32) -> Result<String, String> {
        unsafe {
            let s = vmi_read_str_va(self.vmi, vaddr, pid as _);

            if s == (null::<i8>() as *mut _) {
                Err(format!(
                    "Unable to read string from address 0x{:X} for PID {}",
                    vaddr,
                    pid
                ))
            } else {
                // Allocate a normal rust string
                let c_str = CStr::from_ptr(s).to_string_lossy().into_owned();

                // Free the one allocated by libvmi
                free(s as *mut _);

                Ok(c_str)
            }
        }
    }

    pub fn vmi_read_addr_ksym(&mut self, name: &str) -> Result<addr_t, String> {
        unsafe {
            let mut addr: addr_t = uninitialized();

            if vmi_read_addr_ksym(
                self.vmi,
                CString::new(name).unwrap().as_ptr() as *mut _,
                &mut addr as *mut _,
            ) == status_VMI_FAILURE
            {
                Err(format!("Unable to read addr from symbol {}", name))
            } else {
                Ok(addr)
            }
        }
    }

    pub fn vmi_read_32_ksym(&mut self, name: &str) -> Result<u32, String> {
        unsafe {
            let mut val: u32 = 0;

            if vmi_read_32_ksym(
                self.vmi,
                CString::new(name).unwrap().as_ptr() as *mut _,
                &mut val as *mut _,
            ) == status_VMI_FAILURE
            {
                Err(format!("Unable to read u32 from symbol {}", name))
            } else {
                Ok(val)
            }
        }
    }

    pub fn vmi_read_64_ksym(&mut self, name: &str) -> Result<u64, String> {
        unsafe {
            let mut val: u64 = 0;

            if vmi_read_64_ksym(
                self.vmi,
                CString::new(name).unwrap().as_ptr() as *mut _,
                &mut val as *mut _,
            ) == status_VMI_FAILURE
            {
                Err(format!("Unable to read u64 from symbol {}", name))
            } else {
                Ok(val)
            }
        }
    }

    pub fn vmi_read_str_ksym(&mut self, name: &str) -> Result<String, String> {
        unsafe {
            let s = vmi_read_str_ksym(self.vmi, CString::new(name).unwrap().as_ptr() as *mut _);

            if s == (null::<i8>() as *mut _) {
                Err(format!("Unable to read string from symbol {}", name))
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
        if let Err(msg) = self.vmi_resume_vm() {
            println!("Error while dropping VMI handle: {}", msg);
        }

        // Destroy the handle
        unsafe {
            if vmi_destroy(self.vmi) == status_VMI_FAILURE {
                println!("Unable to destroy handle before dropping");
            }
        }
    }
}
