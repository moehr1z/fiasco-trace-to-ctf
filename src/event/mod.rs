pub mod bp;
pub mod common;
pub mod context_switch;
pub mod destroy;
pub mod drq;
pub mod empty;
pub mod event_type;
pub mod exregs;
pub mod factory;
pub mod fullsize;
pub mod gate;
pub mod ieh;
pub mod ipc;
pub mod ipc_res;
pub mod ipfh;
pub mod irq;
pub mod ke;
pub mod ke_bin;
pub mod ke_reg;
pub mod migration;
pub mod nam;
pub mod pf;
pub mod rcu;
pub mod sched;
pub mod svm;
pub mod timer;
pub mod tmap;
pub mod trap;
pub mod vcpu;

use super::event::{
    bp::BpEvent, common::EventCommon, context_switch::ContextSwitchEvent, destroy::DestroyEvent,
    drq::DrqEvent, empty::EmptyEvent, event_type::EventType, exregs::ExregsEvent,
    factory::FactoryEvent, fullsize::FullsizeEvent, gate::GateEvent, ieh::IehEvent, ipc::IpcEvent,
    ipc_res::IpcResEvent, ipfh::IpfhEvent, irq::IrqEvent, ke::KeEvent, ke_bin::KeBinEvent,
    ke_reg::KeRegEvent, migration::MigrationEvent, nam::NamEvent, pf::PfEvent, rcu::RcuEvent,
    sched::SchedEvent, svm::SvmEvent, timer::TimerEvent, tmap::TmapEvent, trap::TrapEvent,
    vcpu::VcpuEvent,
};
use crate::parser::error;
use binrw::BinRead;
use num_enum::TryFromPrimitiveError;

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

    // TODO this does not actually really get used anywhere, since every event is given an explicit
    // name, which is used for generating the event class. It would be best to remove this function
    // entirely
    pub fn event_type(&self) -> EventType {
        use EventType::*;
        match self {
            _ => Unused,
        }
    }
}

impl From<TryFromPrimitiveError<EventType>> for error::Error {
    fn from(err: TryFromPrimitiveError<EventType>) -> Self {
        error::Error::EventType(err.number)
    }
}
