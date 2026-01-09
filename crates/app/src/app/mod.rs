#[cfg(target_os = "windows")]
pub(crate) mod windows_only_instance;

#[cfg(target_os = "macos")]
pub(crate) mod macos_only_instance;
