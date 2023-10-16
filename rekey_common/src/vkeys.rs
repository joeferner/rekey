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

        // see https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
        insert(&mut t, "backspace", KeyboardAndMouse::VK_BACK);
        insert(&mut t, "tab", KeyboardAndMouse::VK_TAB);
        insert(&mut t, "clear", KeyboardAndMouse::VK_CLEAR);
        insert(&mut t, "enter", KeyboardAndMouse::VK_RETURN);
        insert(&mut t, "shift", KeyboardAndMouse::VK_SHIFT);
        insert(&mut t, "ctrl", KeyboardAndMouse::VK_CONTROL);
        insert(&mut t, "alt", KeyboardAndMouse::VK_MENU);
        insert(&mut t, "pause", KeyboardAndMouse::VK_PAUSE);
        insert(&mut t, "caps_lock", KeyboardAndMouse::VK_CAPITAL);
        insert(&mut t, "esc", KeyboardAndMouse::VK_ESCAPE);
        insert(&mut t, "space", KeyboardAndMouse::VK_SPACE);
        insert(&mut t, "page_up", KeyboardAndMouse::VK_PRIOR);
        insert(&mut t, "page_down", KeyboardAndMouse::VK_NEXT);
        insert(&mut t, "end", KeyboardAndMouse::VK_END);
        insert(&mut t, "home", KeyboardAndMouse::VK_HOME);
        insert(&mut t, "left", KeyboardAndMouse::VK_LEFT);
        insert(&mut t, "up", KeyboardAndMouse::VK_UP);
        insert(&mut t, "right", KeyboardAndMouse::VK_RIGHT);
        insert(&mut t, "down", KeyboardAndMouse::VK_DOWN);
        insert(&mut t, "select", KeyboardAndMouse::VK_SELECT);
        insert(&mut t, "print", KeyboardAndMouse::VK_PRINT);
        insert(&mut t, "execute", KeyboardAndMouse::VK_EXECUTE);
        insert(&mut t, "print_screen", KeyboardAndMouse::VK_SNAPSHOT);
        insert(&mut t, "insert", KeyboardAndMouse::VK_INSERT);
        insert(&mut t, "delete", KeyboardAndMouse::VK_DELETE);
        insert(&mut t, "help", KeyboardAndMouse::VK_HELP);
        for d in '0'..='9' {
            let d_num = d as u16 - '0' as u16;
            insert(
                &mut t,
                &d.to_string(),
                VIRTUAL_KEY(KeyboardAndMouse::VK_A.0 + d_num),
            );
        }
        for d in 'a'..='z' {
            let d_num = d as u16 - 'a' as u16;
            insert(
                &mut t,
                &d.to_string(),
                VIRTUAL_KEY(KeyboardAndMouse::VK_A.0 + d_num),
            );
        }
        insert(&mut t, "windows", KeyboardAndMouse::VK_LWIN);
        insert(&mut t, "rwindows", KeyboardAndMouse::VK_RWIN);
        insert(&mut t, "sleep", KeyboardAndMouse::VK_SLEEP);
        for d in '0'..='9' {
            let d_num = d as u16 - '0' as u16;
            let s = format!("numpad{}", d.to_string());
            insert(
                &mut t,
                &s,
                VIRTUAL_KEY(KeyboardAndMouse::VK_NUMPAD0.0 + d_num),
            );
        }
        insert(&mut t, "multiply", KeyboardAndMouse::VK_MULTIPLY);
        insert(&mut t, "add", KeyboardAndMouse::VK_ADD);
        insert(&mut t, "separator", KeyboardAndMouse::VK_SEPARATOR);
        insert(&mut t, "subtract", KeyboardAndMouse::VK_SUBTRACT);
        insert(&mut t, "decimal", KeyboardAndMouse::VK_DECIMAL);
        insert(&mut t, "divide", KeyboardAndMouse::VK_DIVIDE);
        for d in 1..=24 {
            let s = format!("f{}", d.to_string());
            insert(
                &mut t,
                &s,
                VIRTUAL_KEY(KeyboardAndMouse::VK_F1.0 + d),
            );
        }
        insert(&mut t, "num_lock", KeyboardAndMouse::VK_NUMLOCK);
        insert(&mut t, "scroll_lock", KeyboardAndMouse::VK_SCROLL);
        insert(&mut t, "lshift", KeyboardAndMouse::VK_LSHIFT);
        insert(&mut t, "rshift", KeyboardAndMouse::VK_RSHIFT);
        insert(&mut t, "lcontrol", KeyboardAndMouse::VK_LCONTROL);
        insert(&mut t, "rcontrol", KeyboardAndMouse::VK_RCONTROL);
        insert(&mut t, "lalt", KeyboardAndMouse::VK_LMENU);
        insert(&mut t, "ralt", KeyboardAndMouse::VK_RMENU);
        insert(&mut t, "browser_back", KeyboardAndMouse::VK_BROWSER_BACK);
        insert(&mut t, "browser_forward", KeyboardAndMouse::VK_BROWSER_FORWARD);
        insert(&mut t, "browser_refresh", KeyboardAndMouse::VK_BROWSER_REFRESH);
        insert(&mut t, "browser_stop", KeyboardAndMouse::VK_BROWSER_STOP);
        insert(&mut t, "browser_search", KeyboardAndMouse::VK_BROWSER_SEARCH);
        insert(&mut t, "browser_favorites", KeyboardAndMouse::VK_BROWSER_FAVORITES);
        insert(&mut t, "browser_home", KeyboardAndMouse::VK_BROWSER_HOME);
        insert(&mut t, "volume_mute", KeyboardAndMouse::VK_VOLUME_MUTE);
        insert(&mut t, "volume_down", KeyboardAndMouse::VK_VOLUME_DOWN);
        insert(&mut t, "volume_up", KeyboardAndMouse::VK_VOLUME_UP);
        insert(&mut t, "media_next_track", KeyboardAndMouse::VK_MEDIA_NEXT_TRACK);
        insert(&mut t, "media_prev_track", KeyboardAndMouse::VK_MEDIA_PREV_TRACK);
        insert(&mut t, "media_stop", KeyboardAndMouse::VK_MEDIA_STOP);
        insert(&mut t, "media_play_pause", KeyboardAndMouse::VK_MEDIA_PLAY_PAUSE);
        insert(&mut t, "launch_mail", KeyboardAndMouse::VK_LAUNCH_MAIL);
        insert(&mut t, "launch_media_select", KeyboardAndMouse::VK_LAUNCH_MEDIA_SELECT);
        insert(&mut t, "launch_app_1", KeyboardAndMouse::VK_LAUNCH_APP1);
        insert(&mut t, "launch_app_2", KeyboardAndMouse::VK_LAUNCH_APP2);
        insert(&mut t, "play", KeyboardAndMouse::VK_PLAY);
        insert(&mut t, "zoom", KeyboardAndMouse::VK_ZOOM);
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
