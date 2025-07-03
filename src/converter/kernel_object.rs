#[derive(Debug)]
pub struct BaseKernelObject {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub enum KernelObject {
    Generic(BaseKernelObject),
    Thread(ThreadObject),
}

impl KernelObject {
    pub fn id(&self) -> &str {
        match self {
            KernelObject::Generic(obj) => &obj.id,
            KernelObject::Thread(obj) => &obj.base.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            KernelObject::Generic(obj) => &obj.name,
            KernelObject::Thread(obj) => &obj.base.name,
        }
    }

    pub fn set_id(&mut self, id: String) {
        match self {
            KernelObject::Generic(obj) => obj.id = id,
            KernelObject::Thread(obj) => obj.base.id = id,
        }
    }

    pub fn set_name(&mut self, name: String) {
        match self {
            KernelObject::Generic(obj) => obj.name = name.into(),
            KernelObject::Thread(obj) => obj.base.name = name.into(),
        }
    }
}

#[derive(Debug)]
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
