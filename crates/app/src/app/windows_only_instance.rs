use std::io;

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::HSTRING;

pub struct SingleInstance {
    _mutex_handle: MutexHandle
}

struct MutexHandle(HANDLE);

impl Drop for MutexHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

impl SingleInstance {
    fn try_lock() -> io::Result<Option<Self>> {
        let mutex_name = format!("{}-Instance-Mutex", env!("CARGO_PKG_NAME"));

        let handle = unsafe {
            CreateMutexW(None, false, &HSTRING::from(mutex_name))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
        };

        let is_first = unsafe { windows::Win32::Foundation::GetLastError() != ERROR_ALREADY_EXISTS };

        if is_first {
            Ok(Some(SingleInstance {
                _mutex_handle: MutexHandle(handle)
            }))
        } else {
            // Release the mutex handle if another instance already exists
            unsafe {
                let _ = CloseHandle(handle);
            }
            Ok(None)
        }
    }
}

pub fn handle_single_instance() -> bool {
    match SingleInstance::try_lock() {
        Ok(Some(_instance)) => {
            // Keep the lock alive until program exits
            std::mem::forget(_instance);
            true
        },
        Ok(None) => false,
        Err(e) => {
            eprintln!("Warning: Failed to check single instance: {}", e);
            true
        }
    }
}
