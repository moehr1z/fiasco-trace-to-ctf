#[derive(Debug, Clone)]
pub struct BaseKernelObject {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum KernelObject {
    Generic(BaseKernelObject),
    Thread(ThreadObject),
    Gate(GateObject),
}

impl KernelObject {
    pub fn id(&self) -> &str {
        match self {
            KernelObject::Generic(obj) => &obj.id,
            KernelObject::Thread(obj) => &obj.base.id,
            KernelObject::Gate(obj) => &obj.base.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            KernelObject::Generic(obj) => &obj.name,
            KernelObject::Thread(obj) => &obj.base.name,
            KernelObject::Gate(obj) => &obj.base.name,
        }
    }

    pub fn set_id(&mut self, id: String) {
        match self {
            KernelObject::Generic(obj) => obj.id = id,
            KernelObject::Thread(obj) => obj.base.id = id,
            KernelObject::Gate(obj) => obj.base.id = id,
        }
    }

    pub fn set_name(&mut self, name: String) {
        match self {
            KernelObject::Generic(obj) => obj.name = name.into(),
            KernelObject::Thread(obj) => obj.base.name = name.into(),
            KernelObject::Gate(obj) => obj.base.name = name.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GateObject {
    pub base: BaseKernelObject,
    pub thread: u64,
}

#[derive(Debug, Clone)]
pub struct ThreadObject {
    pub base: BaseKernelObject,
    pub state: ThreadState,
    pub prio: u64,
}

#[derive(Copy, Clone, Debug)]
pub enum ThreadState {
    Running,
    Blocked,
}
