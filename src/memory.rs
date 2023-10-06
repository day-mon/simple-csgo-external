// use std::alloc::System;
// use std::os::windows::raw::HANDLE;
// use winapi::um::handleapi::CloseHandle;
// use crate::runtime::SystemProcess;
//
// pub struct CSGORuntime {

//     process: SystemProcess
// }
//
// impl CSGORuntime {
//     pub fn new(process: SystemProcess) -> Self {
//         Self {
//             process
//         }
//     }
//
//     pub fn get_address(&self, module: &str, offset: usize) -> usize {
//         let module = match module {
//             // "client" => self.client,
//             // "engine" => self.engine,
//             // "panorama" => self.panorama,
//             _ => panic!("Invalid module name")
//         };
//
//         // module + offset
//     }
// }
//
//
