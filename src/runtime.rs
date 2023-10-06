// use std::{mem, ptr};
// use std::collections::HashMap;
// use std::mem::size_of;
// use std::thread::sleep;
// use sysinfo::{PidExt, Process, ProcessExt, Signal, SystemExt};
// use winapi::shared::basetsd::{DWORD_PTR, SIZE_T};
//
// use winapi::shared::minwindef::{DWORD, FALSE, HMODULE, LPCVOID, LPVOID};
// use winapi::um::winnt::PROCESS_ALL_ACCESS;
// use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
// use winapi::shared::ntdef::{HANDLE, NULL};
// use winapi::um::handleapi::CloseHandle;
// use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
// use winapi::um::psapi::{EnumProcessModulesEx, GetModuleFileNameExW, LIST_MODULES_32BIT, LIST_MODULES_64BIT};
//
// pub struct Module {
//     name: String,
//     module: HMODULE,
// }
//
// impl Module {
//     fn name(&self) -> &str {
//         &self.name
//     }
//     pub fn new(name: String, module: HMODULE) -> Self {
//         Self {
//             name,
//             module,
//         }
//     }
// }
//
// pub struct SystemProcess {
//     pid: u32,
//     handle: HANDLE,
//     name: String,
//     modules: HashMap<String, Module>,
// }
//
// impl SystemProcess {
//     pub fn    new(self, pid: u32, handle: HANDLE, name: String) -> Self {
//         let modules = ["client.dll", "panorama.dll", "engine.dll"]
//             .iter()
//             .map(|module| self.get_module_from_process(module).unwrap_or_else(|| {
//                 println!("Failed to get module {}", module);
//                 std::process::exit(1)
//             }))
//             .fold(HashMap::new(), |mut acc, module| {
//                 acc.insert(module.name().to_string(), module);
//                 acc
//             });
//
//         Self {
//             pid,
//             handle,
//             name,
//             modules,
//         }
//     }
//
//     pub fn pid(&self) -> u32 {
//         self.pid
//     }
//
//     pub fn handle(&self) -> HANDLE {
//         self.handle
//     }
//
//     pub fn name(&self) -> &str {
//         &self.name
//     }
//
//     pub fn modules(&self) -> &HashMap<String, Module> {
//         &self.modules
//     }
//
//     pub fn add_module(&mut self, name: String, module: Module) {
//         self.modules.insert(name, module);
//     }
//
//     pub fn get_module(&self, name: &str) -> Option<&Module> {
//         self.modules.get(name)
//     }
//
//     pub fn get_module_from_process(&self, name: &str) -> Option<Module> {
//         let mut mod_list: [HMODULE; 1024] = [NULL as HMODULE; 1024];
//         let mut cur_mod: [u16; 260] = [0; 260];
//         let mut mod_cnt: DWORD = 0;
//
//         if unsafe { EnumProcessModulesEx(self.handle, mod_list.as_mut_ptr(), size_of::<[DWORD; 1024]>() as u32, &mut mod_cnt, LIST_MODULES_32BIT | LIST_MODULES_64BIT) == 0 } {
//             println!("ERROR!");
//             std::process::exit(1)
//         }
//
//         for module in mod_list {
//             unsafe { GetModuleFileNameExW(self.handle, module, cur_mod.as_mut_ptr(), 260); };
//             let mod_str = String::from_utf16(&cur_mod).unwrap();
//             if mod_str.contains(name) {
//                 return Some(Module::new(name.to_string().replace(".dll", ""), module));
//             }
//         }
//
//         None
//     }
// }
//
// impl Drop for SystemProcess {
//     fn drop(&mut self) {
//         unsafe {
//             CloseHandle(self.handle);
//         }
//     }
// }
//
// fn process_from_name(name: &str) -> Option<SystemProcess> {
//     let mut system = sysinfo::System::new();
//     system.refresh_all();
//
//     let process = system.processes()
//         .values()
//         .find(|process| process.name() == name)?;
//
//     let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, process.pid().as_u32()) };
//
//     if handle == NULL {
//         unsafe { CloseHandle(handle) };
//         return None;
//     }
//
//
//     Some(SystemProcess::new(/* SystemProcess */, process.pid().as_u32(), handle, process.name().to_string()))
// }
