use log::error;
use num_enum::TryFromPrimitive;

const IPCWAIT: u8 = IpcType::OpenWait as u8 | IpcType::Recv as u8;
const IPCSENDANDWAIT: u8 = IpcType::OpenWait as u8 | IpcType::Send as u8 | IpcType::Recv as u8;
const IPCREPLYANDWAIT: u8 =
    IpcType::OpenWait as u8 | IpcType::Send as u8 | IpcType::Recv as u8 | IpcType::Reply as u8;
const IPCCALLIPC: u8 = IpcType::Send as u8 | IpcType::Recv as u8;
const SENDRCAP: u8 = IpcType::Send as u8 | IpcType::Reply as u8;

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum IpcType {
    Call = 0,
    Send = 1,
    Recv = 2,
    OpenWait = 4,
    Reply = 8,
    Wait = IPCWAIT,
    SendAndWait = IPCSENDANDWAIT,
    ReplyAndWait = IPCREPLYANDWAIT,
    CallIpc = IPCCALLIPC,
    SendRcap = SENDRCAP,
    Unk,
}

impl IpcType {
    pub fn num_to_str(type_number: u8) -> String {
        let type_var: IpcType = type_number.try_into().unwrap_or_else(|_| {
            error!("Unknown IPC type number {type_number}");
            IpcType::Unk
        });

        format!("{:?}", type_var).to_string()
    }
}
