use ctf_macros::CtfEventClass;

#[derive(CtfEventClass)]
#[event_name_from_event_type]
#[allow(unused)]
pub struct Unsupported {
    // No payload fields
}
