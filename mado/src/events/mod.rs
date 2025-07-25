use serde::Serialize;

#[derive(Serialize)]
pub struct Event<T> {
    pub kind: &'static str,
    pub value: T,
}

pub trait EventRaiser {
    fn raise_event<T: Serialize>(&self, event: Event<T>);
}
