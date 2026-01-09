#[cfg(target_os = "windows")]
pub(crate) mod windows_only_instance;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "freebsd"))]
pub(crate) mod unix_only_instance;
