use std::mem::MaybeUninit;
use std::sync::mpsc::Sender;
use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Events {
    RightClickTrayIcon,
    DoubleClickTrayIcon,
    Open,
    Exit,
}

pub struct Tray {
    tray_icon: TrayIcon<Events>,
}

impl Tray {
    pub fn init(sender: Sender<Events>) -> Tray {
        let icon = include_bytes!("../tray.ico");
        let tray_icon = TrayIconBuilder::new()
            .sender(move |e: &Events| {
                let _ = sender.send(*e);
            })
            .on_right_click(Events::RightClickTrayIcon)
            .on_double_click(Events::DoubleClickTrayIcon)
            .menu(
                MenuBuilder::new()
                    .item("Open", Events::Open)
                    .item("Exit", Events::Exit)
            )
            .icon_from_buffer(icon)
            .build()
            .unwrap();
        Tray { tray_icon }
    }

    pub fn handle_win_messages() {
        loop {
            unsafe {
                let mut msg = MaybeUninit::uninit();
                let bret = GetMessageA(msg.as_mut_ptr(), HWND::default(), 0, 0);
                if bret.0 > 0 {
                    TranslateMessage(msg.as_ptr());
                    DispatchMessageA(msg.as_ptr());
                } else {
                    break;
                }
            }
        }
    }
    pub fn show_menu(&mut self) {
        self.tray_icon.show_menu().unwrap()
    }
}