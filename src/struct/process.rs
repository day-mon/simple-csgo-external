use std::os::windows::raw::HANDLE;
use sysinfo::{PidExt, ProcessExt, SystemExt};
use winapi::shared::minwindef::FALSE;
use winapi::shared::ntdef::NULL;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_ALL_ACCESS;

pub struct Module {
    name: String
}

pub struct Process {
    handle: HANDLE,
    modules: Vec<Module>,
}


impl Process {
    pub fn new(handle: HANDLE, modules: Vec<Module>) -> Self {
        Self {
            handle,
            modules,
        }
    }

    fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|module| module.name == name)
    }


    pub fn from_name(name: &str) -> Option<Self> {
        let mut system = sysinfo::System::new();
        system.refresh_all();


        let Some(process) = system.processes()
            .values()
            .find(|process| process.name() == name) else {
            println!("{name} not found");
            std::process::exit(1)
        };

        let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, process.pid().as_u32()) };
        if process_handle == NULL {
            unsafe { CloseHandle(process_handle) };
            println!("OpenProcess failed");
            std::process::exit(1)
        }

        Some(Self::new(process_handle, vec![]))
    }
}


