pub enum EventType {
    CookieRead,
    Finish,
}

pub struct CookieReadEvent {
    pub m_type: EventType,
}
