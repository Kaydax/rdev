use crate::rdev::{Button, EventType, SimulateError};
use crate::windows::keyboard::UINT;
use crate::windows::keycodes::{code_from_key, scan_from_code};
use crate::windows::{DWORD, LONG, MOUSE_BACKWARD, MOUSE_FORWARD, WORD};
use std::convert::TryFrom;
use std::ffi::c_int;
use std::mem::size_of;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN,
    MOUSEEVENTF_XUP, MOUSEINPUT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, WHEEL_DELTA,
};
/// Not defined in win32 but define here for clarity
static KEYEVENTF_KEYDOWN: u32 = 0;

fn sim_mouse_event(flags: DWORD, data: LONG, dx: LONG, dy: LONG) -> Result<(), SimulateError> {
    let mut union: INPUT_0 = unsafe { std::mem::zeroed() };
    let inner_union = unsafe { &mut union.mi };
    *inner_union = MOUSEINPUT {
        dx,
        dy,
        mouseData: data,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    let mut input = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: union,
    }; 1];
    let value = unsafe {
        SendInput(
            input.len() as UINT,
            input.as_mut_ptr(),
            size_of::<INPUT>() as c_int,
        )
    };
    if value != 1 {
        Err(SimulateError)
    } else {
        Ok(())
    }
}

fn sim_keyboard_event(flags: DWORD, vk: WORD, scan: WORD) -> Result<(), SimulateError> {
    let mut union: INPUT_0 = unsafe { std::mem::zeroed() };
    let inner_union = unsafe { &mut union.ki };
    *inner_union = KEYBDINPUT {
        wVk: vk,
        wScan: scan,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    let mut input = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: union,
    }; 1];
    let value = unsafe {
        SendInput(
            input.len() as UINT,
            input.as_mut_ptr(),
            size_of::<INPUT>() as c_int,
        )
    };
    if value != 1 {
        Err(SimulateError)
    } else {
        Ok(())
    }
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    match event_type {
        EventType::KeyPress(key) => {
            let code = code_from_key(*key).ok_or(SimulateError)?;
            let scan = scan_from_code(code).ok_or(SimulateError)?;
            sim_keyboard_event(KEYEVENTF_KEYDOWN, code, scan)
        }
        EventType::KeyRelease(key) => {
            let code = code_from_key(*key).ok_or(SimulateError)?;
            let scan = scan_from_code(code).ok_or(SimulateError)?;
            sim_keyboard_event(KEYEVENTF_KEYUP, code, scan)
        }
        EventType::ButtonPress(button) => match button {
            Button::Left => sim_mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
            Button::Middle => sim_mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0),
            Button::Right => sim_mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0),
            Button::Forward => sim_mouse_event(MOUSEEVENTF_XDOWN, MOUSE_FORWARD.into(), 0, 0),
            Button::Backward => sim_mouse_event(MOUSEEVENTF_XDOWN, MOUSE_BACKWARD.into(), 0, 0),
            Button::Unknown(code) => sim_mouse_event(MOUSEEVENTF_XDOWN, (*code).into(), 0, 0),
        },
        EventType::ButtonRelease(button) => match button {
            Button::Left => sim_mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0),
            Button::Middle => sim_mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0),
            Button::Right => sim_mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0),
            Button::Forward => sim_mouse_event(MOUSEEVENTF_XUP, MOUSE_FORWARD.into(), 0, 0),
            Button::Backward => sim_mouse_event(MOUSEEVENTF_XUP, MOUSE_BACKWARD.into(), 0, 0),
            Button::Unknown(code) => sim_mouse_event(MOUSEEVENTF_XUP, (*code).into(), 0, 0),
        },
        EventType::Wheel { delta_x, delta_y } => {
            if *delta_x != 0 {
                sim_mouse_event(
                    MOUSEEVENTF_HWHEEL,
                    (u32::try_from(*delta_x).map_err(|_| SimulateError)? * WHEEL_DELTA) as LONG,
                    0,
                    0,
                )?;
            }

            if *delta_y != 0 {
                sim_mouse_event(
                    MOUSEEVENTF_WHEEL,
                    (u32::try_from(*delta_y).map_err(|_| SimulateError)? * WHEEL_DELTA) as LONG,
                    0,
                    0,
                )?;
            }
            Ok(())
        }
        EventType::MouseMove { x, y } => {
            let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
            let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
            if width == 0 || height == 0 {
                return Err(SimulateError);
            }

            sim_mouse_event(
                MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                0,
                (*x as i32 + 1) * 65535 / width,
                (*y as i32 + 1) * 65535 / height,
            )
        }
    }
}
