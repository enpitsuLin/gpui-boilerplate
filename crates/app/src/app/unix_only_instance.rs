use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

pub struct SingleInstance {
    _lock_file: File,
}

impl SingleInstance {
    fn try_lock() -> io::Result<Option<Self>> {
        let lock_path = Self::get_lock_file_path()?;

        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)?;

        // Try to acquire exclusive lock (non-blocking)
        let result = unsafe {
            libc::flock(
                lock_file.as_raw_fd(),
                libc::LOCK_EX | libc::LOCK_NB,
            )
        };

        if result == 0 {
            Ok(Some(SingleInstance {
                _lock_file: lock_file,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_lock_file_path() -> io::Result<PathBuf> {
        let bundle_id = env!("CARGO_PKG_NAME");

        let home_dir = std::env::var("HOME")
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;

        #[cfg(target_os = "macos")]
        let lock_path = {
            let mut path = PathBuf::from(&home_dir);
            path.push("Library");
            path.push("Application Support");
            path.push(bundle_id);
            path.push("instance.lock");
            path
        };

        #[cfg(not(target_os = "macos"))]
        let lock_path = {
            // Use XDG Base Directory specification for Linux/BSD
            let xdg_data_home = std::env::var("XDG_DATA_HOME")
                .unwrap_or_else(|_| format!("{}/.local/share", home_dir));

            let mut path = PathBuf::from(xdg_data_home);
            path.push(bundle_id);
            path.push("instance.lock");
            path
        };

        Ok(lock_path)
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self._lock_file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

pub fn handle_single_instance() -> bool {
    match SingleInstance::try_lock() {
        Ok(Some(_instance)) => {
            // Keep the lock alive until program exits
            std::mem::forget(_instance);
            true
        }
        Ok(None) => false,
        Err(e) => {
            eprintln!("Warning: Failed to check single instance: {}", e);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_file_path() {
        let path = SingleInstance::get_lock_file_path().unwrap();
        let path_str = path.to_str().unwrap();

        #[cfg(target_os = "macos")]
        {
            assert!(path_str.contains("Library/Application Support"));
        }

        #[cfg(not(target_os = "macos"))]
        {
            assert!(path_str.contains(".local/share") || path_str.contains("XDG_DATA_HOME"));
        }

        assert!(path_str.contains(env!("CARGO_PKG_NAME")));
        assert!(path_str.ends_with("instance.lock"));
    }

    #[test]
    fn test_single_instance() {
        let instance1 = SingleInstance::try_lock().unwrap();
        assert!(instance1.is_some());

        let instance2 = SingleInstance::try_lock().unwrap();
        assert!(instance2.is_none());

        drop(instance1);

        let instance3 = SingleInstance::try_lock().unwrap();
        assert!(instance3.is_some());
    }
}
