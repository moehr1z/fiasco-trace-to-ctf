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

#[derive(Debug)]
pub struct ThreadObject {
    pub base: BaseKernelObject,
    pub state: ThreadState,
    pub prio: u64,
}

#[derive(Debug)]
pub enum ThreadState {
    Running,
    Blocked,
}
