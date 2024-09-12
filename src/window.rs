use std::{
    error::Error,
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    ptr::null_mut
};

use windows::Win32::{
    Foundation::{HMODULE, HWND},
    System::{
        ProcessStatus::GetModuleFileNameExW,
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
    UI::WindowsAndMessaging::{
        GetWindowTextW,
        GetWindowThreadProcessId,
    },
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

pub struct Window {
    handle: HWND,
}

impl Window {
    pub fn get_active() -> Window {
        Window {
            handle: unsafe { GetForegroundWindow() },
        }
    }

    fn get_title(&self) -> Result<String, Box<dyn Error>> {
        let mut buffer = [0; 0x400];
        let title_length = unsafe { GetWindowTextW(self.handle, &mut buffer) };
        if title_length == 0 {
            return Err("Failed to get window title".into());
        }
        Ok(String::from_utf16(&buffer[..title_length as usize])?)
    }

    fn get_exec_path(&self) -> Result<String, Box<dyn Error>> {
        let mut process_id = 0;
        if unsafe { GetWindowThreadProcessId(self.handle, Some(&mut process_id)) } == 0 {
            return Err("Could not get process id".into());
        }
        let process = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                process_id,
            )
        }?;
        if process.is_invalid() {
            return Err("Failed to open process".into());
        }
        let mut buffer: Vec<u16> = vec![0; 1024];
        let len =
            unsafe { GetModuleFileNameExW(process, HMODULE(null_mut()), &mut buffer) as usize };
        if len == 0 {
            return Err("Failed to get module file name".into());
        }

        let file_name = OsString::from_wide(&buffer[..len]);
        Ok(file_name.to_string_lossy().into_owned())
    }

    pub fn get_display_name(&self) -> Option<String> {
        let exec_path = self
            .get_exec_path()
            .map_err(|err| {
                println!("Could not get exec path: {}", err);
            })
            .ok();
        exec_path.as_ref()?;
        let exec_path_unwrapped = exec_path.unwrap();
        let path_buf = PathBuf::from(&exec_path_unwrapped);
        let exe_name = path_buf
            .file_name()
            .and_then(|file_name| file_name.to_str());
        if exe_name.is_none() {
            println!("Could not get file name from path: {}", exec_path_unwrapped);
            return None;
        }
        if exe_name.unwrap().eq("ApplicationFrameHost.exe") {
            self.get_title()
                .map_err(|err| {
                    println!("Could not get window title: {}", err);
                })
                .ok()
        } else {
            Some(String::from(exe_name.unwrap()))
        }
    }
}
