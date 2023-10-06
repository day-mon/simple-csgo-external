use std::ops::Not;
use winapi::um::winuser::{GetAsyncKeyState, GetForegroundWindow, mouse_event, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, VK_CONTROL, VK_LMENU};
use crate::r#struct::cheat::Cheat;
use crate::r#struct::context::PlayerEntity;

pub(crate) struct Trigger;

impl Cheat for Trigger {
    fn toggle(local_player: &PlayerEntity, entities: &[PlayerEntity])  {
        if ! local_player.is_aiming_at_valid_enemy(entities) {
            println!("Not aiming at valid enemy");
            return;
        }

        if ! local_player.on_ground() {
            println!("Not on ground");
            return;
        }

        if unsafe { GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0 } {
            unsafe { mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0); }
            unsafe { mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0); }
        }
    }
}