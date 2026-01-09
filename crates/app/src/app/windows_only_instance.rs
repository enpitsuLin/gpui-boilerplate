use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::HSTRING;

#[inline]
fn is_first_instance() -> bool {
    let mutex_name = format!("{}-Instance-Mutex", env!("CARGO_PKG_NAME"));
    unsafe {
        CreateMutexW(None, false, &HSTRING::from(mutex_name))
            .expect("Unable to create instance mutex.")
    };
    unsafe { GetLastError() != ERROR_ALREADY_EXISTS }
}

pub fn handle_single_instance() -> bool {
    let is_first_instance = is_first_instance();
    if is_first_instance {
        // TODO: handle single instance listener
    }
    is_first_instance
}
