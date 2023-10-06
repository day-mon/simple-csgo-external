mod memory;
mod runtime;
mod r#struct;
mod cheats;
mod constants;


use std::{mem, ptr};

use std::io::Write;
use std::mem::size_of;
use std::process::exit;
use std::time::Duration;
use sysinfo::{PidExt, ProcessExt, Signal, SystemExt};

use winapi::shared::minwindef::{DWORD, FALSE, HMODULE, LPVOID};
use winapi::um::winnt::PROCESS_ALL_ACCESS;
use winapi::um::processthreadsapi::{OpenProcess};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleFileNameExW, LIST_MODULES_64BIT, LIST_MODULES_ALL};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Module32First, MODULEENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32};
use crate::r#struct::cheat::Cheat;

use crate::r#struct::context::{GameState, PlayerEntity, PlayerEntityRaw};
use crate::r#struct::offsets::Offsets;

const OFFSETS_URL: &str = "https://raw.githubusercontent.com/frk1/hazedumper/master/csgo.json";

#[tokio::main]
async fn main() {
    let mut system = sysinfo::System::new();
    system.refresh_all();


    let mut retries = 0;

    while retries < 5 {
        let failed = system
            .processes()
            .values()
            .filter(|val| val.name().to_lowercase().contains("faceit"))
            .map(|process| process.kill_with(Signal::Kill).unwrap_or(false))
            .fold(0, |acc, val| acc + (!val as u32));

        if failed == 0 {
            break;
        }

        retries += 1;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    let Some(csgo_p) = system.processes()
        .values()
        .find(|process| process.name() == "cs2.exe") else {
        println!("CS2 not found");
        exit(1)
    };



    let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, csgo_p.pid().as_u32()) };
    if process_handle == NULL {
        unsafe { CloseHandle(process_handle) };
        println!("OpenProcess failed");
        exit(1)
    }

    println!("Process handle: {:?}", process_handle);

    let client = get_module(process_handle, "client.dll");
    let engine = get_module(process_handle, "engine.dll");


    if engine.is_none() || client.is_none() {
        println!("Failed to find engine or client");
        std::process::exit(1)
    }

    let client = client.unwrap();
    let engine = engine.unwrap();




    let offsets = reqwest::get(OFFSETS_URL)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let Ok(offsets) = serde_json::from_str::<Offsets>(&offsets) else {
        println!("Failed to parse offsets");
        exit(1)
    };

    loop {
        let game_state_loc = rpm(process_handle, engine + offsets.signatures.dw_client_state);
        let game_state = rpm(process_handle, game_state_loc + offsets.signatures.dw_client_state_state);

        let actual_state = GameState::from_u32(game_state);


        if actual_state != GameState::FullConnected {
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        let local_player_ptr = rpm(process_handle, client + offsets.signatures.dw_local_player);
        let entity = rmpc(process_handle, local_player_ptr, size_of::<PlayerEntityRaw>());
        let xhair_id = rpm(process_handle, local_player_ptr + offsets.netvars.m_i_crosshair_id);
        let local_player = PlayerEntity::from_raw_vec(&entity, Some(xhair_id));
        let entities: Vec<PlayerEntity> = (1..constants::MAX_PLAYERS)
            .map(|i| {
                let base = client + offsets.signatures.dw_entity_list + i * 0x10;
                let entity = rpm(process_handle, base);
                let entity = rmpc(process_handle, entity, size_of::<PlayerEntityRaw>());
                PlayerEntity::from_raw_vec(&entity, None)
            })
            .collect();

        for entity in entities.iter() {
            if entity.health > 0 {
                println!("{entity}");
            }
        }


        cheats::trigger::Trigger::toggle(&local_player, entities.as_slice());
        tokio::time::sleep(Duration::from_millis(100)).await;

    }
}
pub fn rpm(handle: HANDLE, dw_addr: u32) -> u32 {
    let mut res: u32 = unsafe { mem::zeroed() };
    unsafe {
        ReadProcessMemory(handle, dw_addr as *mut _, &mut res as *mut _ as LPVOID, size_of::<u32>(), ptr::null_mut());
    }
    res
}

pub fn rmpc(handle: HANDLE, dw_addr: u32, size: usize) -> Vec<u8> {
    let mut res: Vec<u8> = vec![0; size];
    unsafe {
        ReadProcessMemory(handle, dw_addr as *mut _, res.as_mut_ptr() as LPVOID, size, ptr::null_mut());
    }
    res
}


pub fn write_mem<T>(process_handle: HANDLE, dw_addr: DWORD, value: &mut T) {
    unsafe { WriteProcessMemory(process_handle, dw_addr as usize as *mut _, value as *mut _ as LPVOID, size_of::<T>(), NULL as *mut usize); };
}

fn get_module(process_handle: HANDLE, module_name: &str) -> Option<u32> {
    let mut mod_list: [HMODULE; 1024] = [NULL as HMODULE; 1024];
    let mut cur_mod: [u16; 260] = [0; 260];
    let mut mod_cnt: DWORD = 0;

    if unsafe { EnumProcessModulesEx(process_handle, mod_list.as_mut_ptr(), size_of::<[DWORD; 1024]>() as u32, &mut mod_cnt,  LIST_MODULES_64BIT) == 0 } {
        let error = std::io::Error::last_os_error();
        println!("ERROR: {error}");
        std::process::exit(-1);
    }


    for module in mod_list {
        unsafe { GetModuleFileNameExW(process_handle, module, cur_mod.as_mut_ptr(), 260); };
        let mod_str = String::from_utf16(&cur_mod).unwrap();
        println!("{:?}", mod_str);
        let dll_name = mod_str.split('\\').last().unwrap();

        if dll_name.starts_with(module_name) {
            return Some(module as u32);
        }
    }

    None
}