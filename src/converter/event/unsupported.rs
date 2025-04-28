use ctf_macros::CtfEventClass;

#[derive(CtfEventClass)]
#[event_name_from_event_type]
pub struct Unsupported {
    // No payload fields
}
