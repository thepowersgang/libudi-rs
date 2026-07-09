//! Shared global state for testing the sinks
pub struct SinkSharedState {
    inner: ::std::sync::Mutex<Inner>,
}
struct Inner {
    // Can't use a HashMap, as it can't be const-constructed (due to having RNG dependency)
    handlers: ::std::collections::BTreeMap<String,Box<dyn SinkHandler>>,
}
pub trait SinkHandler: 'static + Send {
    fn send(&mut self, bytes: &[u8]);
    fn assert_rx(&mut self, bytes: &[u8]);
}

impl SinkSharedState {
    pub const fn new() -> Self {
        Self {
            inner: ::std::sync::Mutex::new(Inner {
                handlers: ::std::collections::BTreeMap::new(),
            })
        }
    }
    pub fn add(&self, name: impl Into<String>, instance: impl SinkHandler) -> Result<(),()> {
        let r = self.inner.lock().unwrap()
            .handlers.insert(name.into(), Box::new(instance))
            ;
        match r {
        None => Ok(()),
        Some(_) => Err(()),
        }
    }
    pub fn with(&self, name: &str, v: impl FnOnce(&mut dyn SinkHandler)) {
        v(&mut **self.inner.lock().unwrap().handlers.get_mut(name).expect("Unable to find named sink"))
    }
}

#[derive(Clone,Default)]
pub struct SharedByteQueue {
    inner: ::std::sync::Arc<::std::sync::Mutex<::std::collections::VecDeque<u8>>>,
}
impl SharedByteQueue {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&self, data: &[u8]) {
        let mut lh = self.inner.lock().unwrap();
        for &b in data {
            lh.push_back(b);
        }
    }
    pub fn assert_rx(&self, exp: &[u8]) {
        let mut lh = self.inner.lock().unwrap();
        for &b in exp {
            assert!(lh.pop_front() == Some(b));
        }
    }
}