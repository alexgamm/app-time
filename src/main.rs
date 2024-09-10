use std::{
    any::Any,
    error::Error,
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    ptr::null_mut,
    thread::sleep,
    time::Duration,
};

use exe::{VSVersionInfo, VecPE};
use windows::{
    core::Param,
    Win32::{
        Foundation::{HMODULE, HWND},
        System::{
            ProcessStatus::{EnumProcessModules, GetModuleFileNameExW},
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
        UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowModuleFileNameW, GetWindowTextA, GetWindowTextW,
            GetWindowThreadProcessId,
        },
    },
};

fn main() {
    loop {
        let window = Window {
            handle: unsafe { GetForegroundWindow() },
        };
        let display_name = window.get_display_name().unwrap();
        println!("{display_name}");
        sleep(Duration::from_secs(1))
    }
}

struct Window {
    handle: HWND,
}
impl Window {
    fn get_window_title(&self) -> Result<String, Box<dyn Error>> {
        let mut buffer = [0; 0x400];
        let title_length = unsafe { GetWindowTextW(self.handle, &mut buffer) };
        if title_length == 0 {
            return Err("Failed to get window title".into());
        }
        Ok(String::from_utf16(&buffer[..title_length as usize])?)
    }

    fn get_window_exec_path(&self) -> Result<String, Box<dyn Error>> {
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

        let mut cb_needed = 0;

        unsafe {
            let mut buffer: Vec<u16> = vec![0; 1024];
            let len =
                unsafe { GetModuleFileNameExW(process, HMODULE(null_mut()), &mut buffer) as usize };
            if len == 0 {
                return Err("Failed to get module file name".into());
            }

            let file_name = OsString::from_wide(&buffer[..len]);
            Ok(file_name.to_string_lossy().into_owned())
        }
    }

    fn get_display_name(&self) -> Option<String> {
        let exec_path = self
            .get_window_exec_path()
            .map_err(|err| {
                println!("Could not get exec path: {}", err);
            })
            .ok();
        if exec_path.is_none() {
            return None;
        }
        let exec_path_unwrapped = exec_path.unwrap();
        let path_buf = PathBuf::from(&exec_path_unwrapped);
        let exe_name = path_buf
            .file_name()
            .map(|file_name| file_name.to_str())
            .flatten();
        if exe_name.is_none() {
            println!("Could not get file name from path: {}", exec_path_unwrapped);
            return None;
        }
        if exe_name.unwrap().eq("ApplicationFrameHost.exe") {
            self.get_window_title()
                .map_err(|err| {
                    println!("Could not get window title: {}", err);
                })
                .ok()
        } else {
            Window::get_exe_description(&exec_path_unwrapped)
        }
    }

    fn get_exe_description(exe_path: &String) -> Option<String> {
        println!("PATH {}", exe_path);
        let pefile = VecPE::from_disk_file(exe_path).unwrap();
        let vs_version = VSVersionInfo::parse(&pefile)
            .unwrap()
            .string_file_info
            .unwrap();
        let description = &vs_version.children[0].string_map().unwrap()["FileDescription"];
        Some(description.to_string())
    }
}
