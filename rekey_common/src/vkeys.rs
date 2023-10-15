use std::collections::HashMap;

use lazy_static::lazy_static;
use windows::Win32::UI::Input::KeyboardAndMouse::{self, VIRTUAL_KEY};

pub struct Vkey {
    pub name: String,
    pub code: VIRTUAL_KEY,
}

impl Clone for Vkey {
    fn clone(&self) -> Self {
        return Self {
            name: self.name.clone(),
            code: self.code.clone(),
        };
    }
}

lazy_static! {
    pub static ref VKEY_LOOKUP_BY_NAME: HashMap<String, Vkey> = {
        let mut t = HashMap::new();

        fn insert(t: &mut HashMap<String, Vkey>, name: &str, code: VIRTUAL_KEY) -> () {
            t.insert(
                name.to_string(),
                Vkey {
                    name: name.to_string(),
                    code,
                },
            );
        }

        insert(&mut t, "esc", KeyboardAndMouse::VK_ESCAPE);
        insert(&mut t, "ctrl", KeyboardAndMouse::VK_CONTROL);
        insert(&mut t, "alt", KeyboardAndMouse::VK_MENU);
        insert(&mut t, "shift", KeyboardAndMouse::VK_SHIFT);
        insert(&mut t, "win", KeyboardAndMouse::VK_LWIN);
        insert(&mut t, "space", KeyboardAndMouse::VK_SPACE);
        insert(&mut t, "backspace", KeyboardAndMouse::VK_BACK);
        insert(&mut t, "tab", KeyboardAndMouse::VK_TAB);
        insert(&mut t, "enter", KeyboardAndMouse::VK_RETURN);
        insert(&mut t, "pause", KeyboardAndMouse::VK_PAUSE);
        insert(&mut t, "left", KeyboardAndMouse::VK_LEFT);
        insert(&mut t, "right", KeyboardAndMouse::VK_RIGHT);
        insert(&mut t, "up", KeyboardAndMouse::VK_UP);
        insert(&mut t, "down", KeyboardAndMouse::VK_DOWN);
        insert(&mut t, "insert", KeyboardAndMouse::VK_INSERT);
        insert(&mut t, "delete", KeyboardAndMouse::VK_DELETE);
        insert(&mut t, "f1", KeyboardAndMouse::VK_F1);
        insert(&mut t, "f2", KeyboardAndMouse::VK_F2);
        insert(&mut t, "f3", KeyboardAndMouse::VK_F3);
        insert(&mut t, "f4", KeyboardAndMouse::VK_F4);
        insert(&mut t, "f5", KeyboardAndMouse::VK_F5);
        insert(&mut t, "f6", KeyboardAndMouse::VK_F6);
        insert(&mut t, "f7", KeyboardAndMouse::VK_F7);
        insert(&mut t, "f8", KeyboardAndMouse::VK_F8);
        insert(&mut t, "f9", KeyboardAndMouse::VK_F9);
        insert(&mut t, "f10", KeyboardAndMouse::VK_F10);
        insert(&mut t, "f11", KeyboardAndMouse::VK_F11);
        insert(&mut t, "f12", KeyboardAndMouse::VK_F12);
        insert(&mut t, "f13", KeyboardAndMouse::VK_F13);
        insert(&mut t, "f14", KeyboardAndMouse::VK_F14);
        insert(&mut t, "f15", KeyboardAndMouse::VK_F15);
        insert(&mut t, "f16", KeyboardAndMouse::VK_F16);
        insert(&mut t, "f17", KeyboardAndMouse::VK_F17);
        insert(&mut t, "f18", KeyboardAndMouse::VK_F18);
        insert(&mut t, "f19", KeyboardAndMouse::VK_F19);
        insert(&mut t, "f20", KeyboardAndMouse::VK_F20);
        insert(&mut t, "f21", KeyboardAndMouse::VK_F21);
        insert(&mut t, "f22", KeyboardAndMouse::VK_F22);
        insert(&mut t, "f23", KeyboardAndMouse::VK_F23);
        insert(&mut t, "f24", KeyboardAndMouse::VK_F24);
        insert(&mut t, "numlock", KeyboardAndMouse::VK_NUMLOCK);
        insert(&mut t, "home", KeyboardAndMouse::VK_HOME);
        insert(&mut t, "end", KeyboardAndMouse::VK_END);
        insert(&mut t, "pageup", KeyboardAndMouse::VK_PRIOR);
        insert(&mut t, "pagedown", KeyboardAndMouse::VK_NEXT);
        insert(&mut t, "clear", KeyboardAndMouse::VK_CLEAR);
        insert(&mut t, "divide", KeyboardAndMouse::VK_DIVIDE);
        insert(&mut t, "multiply", KeyboardAndMouse::VK_MULTIPLY);
        insert(&mut t, "subtract", KeyboardAndMouse::VK_SUBTRACT);
        insert(&mut t, "add", KeyboardAndMouse::VK_ADD);
        insert(&mut t, "launch_app_1", KeyboardAndMouse::VK_LAUNCH_APP1);
        insert(&mut t, "launch_app_2", KeyboardAndMouse::VK_LAUNCH_APP2);

        for d in 'a'..='z' {
            let d_num = d as u16 - 'a' as u16;
            insert(
                &mut t,
                &d.to_string(),
                VIRTUAL_KEY(KeyboardAndMouse::VK_A.0 + d_num),
            );
        }
        for d in '0'..='9' {
            let d_num = d as u16 - '0' as u16;
            insert(
                &mut t,
                &d.to_string(),
                VIRTUAL_KEY(KeyboardAndMouse::VK_A.0 + d_num),
            );
        }
        for d in '0'..='9' {
            let d_num = d as u16 - '0' as u16;
            let s = format!("numpad{}", d.to_string());
            insert(
                &mut t,
                &s,
                VIRTUAL_KEY(KeyboardAndMouse::VK_NUMPAD0.0 + d_num),
            );
        }

        return t;
    };
    pub static ref VKEY_LOOKUP_BY_CODE: HashMap<u16, Vkey> = {
        let mut t: HashMap<u16, Vkey> = HashMap::new();
        for vkey in VKEY_LOOKUP_BY_NAME.values() {
            t.insert(vkey.code.0, vkey.clone());
        }
        return t;
    };
}
