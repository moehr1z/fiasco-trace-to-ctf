/* Note, automatically generated from Fiasco binary */

use core::fmt;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EventType {
    Unused = 0,
    Pf = 1,
    Ipc = 2,
    IpcRes = 3,
    IpcTrace = 4,
    Ke = 5,
    KeReg = 6,
    Breakpoint = 7,
    KeBin = 8,
    ContextSwitch = 9,
    DrqHandling = 10,
    ExRegs = 11,
    ExceptionInvalidHandler = 12,
    Exceptions = 13,
    FactoryDelete = 14,
    IpcGateInvoke = 15,
    IrqObjectTriggers = 16,
    KobjectCreate = 17,
    KobjectDelete = 18,
    KobjectDeleteGeneric = 19,
    KobjectDestroy = 20,
    KobjectNames = 21,
    PageFaultInvalidPager = 22,
    RcuCall = 23,
    RcuCallbacks = 24,
    RcuIdle = 25,
    SchedulingContextLoad = 26,
    SchedulingContextSave = 27,
    TaskMap = 28,
    TaskUnmap = 29,
    ThreadMigration = 30,
    TimerIrqsKernelScheduling = 31,
    VcpuEvents = 32,
    VmSvm = 33,
    Hidden = 128,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EventType::*;
        match self {
            Unused => write!(f, "UNUSED"), 
            Pf => write!(f, "PF"), 
            Ipc => write!(f, "IPC"), 
            IpcRes => write!(f, "IPCRES"), 
            IpcTrace => write!(f, "IPCTRACE"), 
            Ke => write!(f, "KE"), 
            KeReg => write!(f, "KEREG"), 
            Breakpoint => write!(f, "BREAKPOINT"), 
            KeBin => write!(f, "KEBIN"), 
            ContextSwitch => write!(f, "CONTEXTSWITCH"), 
            DrqHandling => write!(f, "DRQHANDLING"), 
            ExRegs => write!(f, "EXREGS"), 
            ExceptionInvalidHandler => write!(f, "EXCEPTIONINVALIDHANDLER"), 
            Exceptions => write!(f, "EXCEPTIONS"), 
            FactoryDelete => write!(f, "FACTORYDELETE"), 
            IpcGateInvoke => write!(f, "IPCGATEINVOKE"), 
            IrqObjectTriggers => write!(f, "IRQOBJECTTRIGGERS"), 
            KobjectCreate => write!(f, "KOBJECTCREATE"), 
            KobjectDelete => write!(f, "KOBJECTDELETE"), 
            KobjectDeleteGeneric => write!(f, "KOBJECTDELETEGENERIC"), 
            KobjectDestroy => write!(f, "KOBJECTDESTROY"), 
            KobjectNames => write!(f, "KOBJECTNAMES"), 
            PageFaultInvalidPager => write!(f, "PAGEFAULTINVALIDPAGER"), 
            RcuCall => write!(f, "RCUCALL"), 
            RcuCallbacks => write!(f, "RCUCALLBACKS"), 
            RcuIdle => write!(f, "RCUIDLE"), 
            SchedulingContextLoad => write!(f, "SCHEDULINGCONTEXTLOAD"), 
            SchedulingContextSave => write!(f, "SCHEDULINGCONTEXTSAVE"), 
            TaskMap => write!(f, "TASKMAP"), 
            TaskUnmap => write!(f, "TASKUNMAP"), 
            ThreadMigration => write!(f, "THREADMIGRATION"), 
            TimerIrqsKernelScheduling => write!(f, "TIMERIRQSKERNELSCHEDULING"), 
            VcpuEvents => write!(f, "VCPUEVENTS"), 
            VmSvm => write!(f, "VMSVM"), 
            Hidden => write!(f, "HIDDEN"), 
        }
    }
}

