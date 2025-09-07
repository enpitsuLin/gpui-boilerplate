use windows::{
    Win32::{
        Foundation::{ERROR_ALREADY_EXISTS, GetLastError},
        System::Threading::CreateMutexW,
    },
    core::HSTRING,
};

#[inline]
fn is_first_instance() -> bool {
    unsafe {
        CreateMutexW(None, false, &HSTRING::from("aria2-gpui-Instance-Mutex"))
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
