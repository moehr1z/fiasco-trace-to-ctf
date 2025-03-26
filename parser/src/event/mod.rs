/* Note, automatically generated from Fiasco binary */

pub mod l4_event;
pub mod drq;
pub mod vcpu;
pub mod factory;
pub mod gate;
pub mod irq;
pub mod destroy;
pub mod nam;
pub mod rcu;
pub mod tmap;
pub mod bp;
pub mod context_switch;
pub mod empty;
pub mod ipc;
pub mod ipc_res;
pub mod ke;
pub mod ke_bin;
pub mod ke_reg;
pub mod pf;
pub mod sched;
pub mod trap;
pub mod fullsize;
pub mod ieh;
pub mod ipfh;
pub mod exregs;
pub mod migration;
pub mod timer;
pub mod svm;

use crate::event::drq::DrqEvent;
use crate::event::vcpu::VcpuEvent;
use crate::event::factory::FactoryEvent;
use crate::event::gate::GateEvent;
use crate::event::irq::IrqEvent;
use crate::event::destroy::DestroyEvent;
use crate::event::nam::NamEvent;
use crate::event::rcu::RcuEvent;
use crate::event::tmap::TmapEvent;
use crate::event::bp::BpEvent;
use crate::event::context_switch::ContextSwitchEvent;
use crate::event::empty::EmptyEvent;
use crate::event::ipc::IpcEvent;
use crate::event::ipc_res::IpcResEvent;
use crate::event::ke::KeEvent;
use crate::event::ke_bin::KeBinEvent;
use crate::event::ke_reg::KeRegEvent;
use crate::event::pf::PfEvent;
use crate::event::sched::SchedEvent;
use crate::event::trap::TrapEvent;
use crate::event::fullsize::FullsizeEvent;
use crate::event::ieh::IehEvent;
use crate::event::ipfh::IpfhEvent;
use crate::event::exregs::ExregsEvent;
use crate::event::migration::MigrationEvent;
use crate::event::timer::TimerEvent;
use crate::event::svm::SvmEvent;

use core::fmt;
use binrw::BinRead;

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};use crate::error;
use l4_event::EventCommon;

pub type L4Addr = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Address = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Cap_index = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Context = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Context__Drq = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Context__Drq_log__Type = u32;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Cpu_number = u32;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Irq_base = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Irq_chip = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Kobject = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__L4_error = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__L4_msg_tag = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__L4_obj_ref = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__L4_timeout_pair = u32;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Mword = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Rcu_item = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Sched_context = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Smword = i64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Space = L4Addr;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Unsigned16 = u16;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Unsigned32 = u32;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Unsigned64 = u64;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__Unsigned8 = u8;
#[allow(non_camel_case_types)]
pub type L4_ktrace_t__cxx__Type_info = L4Addr;


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

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Event {
    Drq(DrqEvent),
    Vcpu(VcpuEvent),
    Factory(FactoryEvent),
    Gate(GateEvent),
    Irq(IrqEvent),
    Destroy(DestroyEvent),
    Nam(NamEvent),
    Rcu(RcuEvent),
    Tmap(TmapEvent),
    Bp(BpEvent),
    ContextSwitch(ContextSwitchEvent),
    Empty(EmptyEvent),
    Ipc(IpcEvent),
    IpcRes(IpcResEvent),
    Ke(KeEvent),
    KeBin(KeBinEvent),
    KeReg(KeRegEvent),
    Pf(PfEvent),
    Sched(SchedEvent),
    Trap(TrapEvent),
    Fullsize(FullsizeEvent),
    Ieh(IehEvent),
    Ipfh(IpfhEvent),
    Exregs(ExregsEvent),
    Migration(MigrationEvent),
    Timer(TimerEvent),
    Svm(SvmEvent),
}

impl Event {
    pub fn event_common(&self) -> EventCommon {
        use Event::*;
        match self {
            Drq(e) => e.common,
            Vcpu(e) => e.common,
            Factory(e) => e.common,
            Gate(e) => e.common,
            Irq(e) => e.common,
            Destroy(e) => e.common,
            Nam(e) => e.common,
            Rcu(e) => e.common,
            Tmap(e) => e.common,
            Bp(e) => e.common,
            ContextSwitch(e) => e.common,
            Empty(e) => e.common,
            Ipc(e) => e.common,
            IpcRes(e) => e.common,
            Ke(e) => e.common,
            KeBin(e) => e.common,
            KeReg(e) => e.common,
            Pf(e) => e.common,
            Sched(e) => e.common,
            Trap(e) => e.common,
            Fullsize(e) => e.common,
            Ieh(e) => e.common,
            Ipfh(e) => e.common,
            Exregs(e) => e.common,
            Migration(e) => e.common,
            Timer(e) => e.common,
            Svm(e) => e.common,
        }
    }

    pub fn event_type(&self) -> EventType {
        use EventType::*;
        match self {
            Event::Drq(_) => Unused,
            Event::Vcpu(_) => Unused,
            Event::Factory(_) => Unused,
            Event::Gate(_) => Unused,
            Event::Irq(_) => Unused,
            Event::Destroy(_) => Unused,
            Event::Nam(_) => Unused,
            Event::Rcu(_) => Unused,
            Event::Tmap(_) => Unused,
            Event::Bp(_) => Unused,
            Event::ContextSwitch(_) => ContextSwitch,
            Event::Empty(_) => Unused,
            Event::Ipc(_) => Ipc,
            Event::IpcRes(_) => IpcRes,
            Event::Ke(_) => Ke,
            Event::KeBin(_) => KeBin,
            Event::KeReg(_) => KeReg,
            Event::Pf(_) => Pf,
            Event::Sched(_) => Unused,
            Event::Trap(_) => Unused,
            Event::Fullsize(_) => Unused,
            Event::Ieh(_) => Unused,
            Event::Ipfh(_) => Unused,
            Event::Exregs(_) => Unused,
            Event::Migration(_) => Unused,
            Event::Timer(_) => Unused,
            Event::Svm(_) => Unused,
        }
    }
}

impl From<TryFromPrimitiveError<EventType>> for error::Error {
    fn from(err: TryFromPrimitiveError<EventType>) -> Self {
        error::Error::EventTypeError(err.number)
    }
}

