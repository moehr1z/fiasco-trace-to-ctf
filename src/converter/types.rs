use babeltrace2_sys::{Error, MessageIteratorStatus, SelfMessageIterator, ffi};
use std::collections::HashMap;
use std::ffi::{CStr, CString};

#[derive(Default)]
pub struct StringCache {
    indices: HashMap<String, usize>,
    strings: Vec<CString>,
}

impl StringCache {
    pub fn insert_str(&mut self, key: &str) -> Result<usize, Error> {
        if let Some(&id) = self.indices.get(key) {
            return Ok(id);
        }

        let cstr = CString::new(key)?;
        let id = self.strings.len();
        self.strings.push(cstr);
        self.indices.insert(key.to_string(), id);
        Ok(id)
    }

    pub fn get_str(&self, key: &str) -> &CStr {
        let id = self.indices.get(key).unwrap_or_else(|| {
            panic!(
                "String cache string entry doesn't exist ({}) INDICES: \n{:?} STRINGS: \n{:?}",
                key, self.indices, self.strings
            )
        });
        self.strings[*id].as_c_str()
    }

    pub fn get_str_by_id(&self, key: usize) -> &CStr {
        self.strings[key].as_c_str()
    }
}

// TODO split up the roles of this, currently just a catch all
pub struct BorrowedCtfState<'a> {
    stream: *mut ffi::bt_stream,
    packet: *mut ffi::bt_packet,
    msg_iter: SelfMessageIterator,
    messages: &'a mut [*const ffi::bt_message],
    msgs_len: usize,
}

impl<'a> BorrowedCtfState<'a> {
    pub fn new(
        stream: *mut ffi::bt_stream,
        packet: *mut ffi::bt_packet,
        msg_iter: SelfMessageIterator,
        messages: &'a mut [*const ffi::bt_message],
    ) -> Self {
        assert!(!stream.is_null());
        assert!(!packet.is_null());
        assert!(!messages.is_empty());
        Self {
            stream,
            packet,
            msg_iter,
            messages,
            msgs_len: 0,
        }
    }

    pub fn release(self) -> MessageIteratorStatus {
        if self.msgs_len == 0 {
            MessageIteratorStatus::NoMessages
        } else {
            MessageIteratorStatus::Messages(self.msgs_len as u64)
        }
    }

    pub fn stream_mut(&mut self) -> *mut ffi::bt_stream {
        self.stream
    }

    pub fn message_iter_mut(&mut self) -> *mut ffi::bt_self_message_iterator {
        self.msg_iter.inner_mut()
    }

    pub fn create_message(
        &mut self,
        event_class: *const ffi::bt_event_class,
        timestamp: u64,
    ) -> *mut ffi::bt_message {
        unsafe {
            ffi::bt_message_event_create_with_packet_and_default_clock_snapshot(
                self.msg_iter.inner_mut(),
                event_class,
                self.packet,
                timestamp,
            )
        }
    }

    pub fn push_message(&mut self, msg: *const ffi::bt_message) -> Result<(), Error> {
        if msg.is_null() {
            Err(Error::PluginError("MessageVec: msg is NULL".to_owned()))
        } else if self.msgs_len >= self.messages.len() {
            Err(Error::PluginError("MessageVec: full".to_owned()))
        } else {
            self.messages[self.msgs_len] = msg;
            self.msgs_len += 1;
            Ok(())
        }
    }
}
