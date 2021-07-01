use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    fmt,
    sync::Arc,
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct Settings {
    options: HashMap<TypeId, Arc<dyn Setting + 'static>>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut default = Self {
            options: HashMap::new(),
        };

        default.insert(Timeout::default());

        default
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_raw(&mut self, any: Arc<dyn Setting + 'static>) {
        self.options.insert(any.id(), any);
    }

    pub fn insert<S>(&mut self, setting: S)
    where
        S: Setting,
    {
        self.insert_raw(Arc::new(setting));
    }

    pub fn get_or_default<S>(&self) -> Arc<S>
    where
        S: Setting + Default,
    {
        self.get::<S>().unwrap_or_else(|| Arc::new(S::default()))
    }

    pub fn get<S>(&self) -> Option<Arc<S>>
    where
        S: Setting,
    {
        let any: Arc<dyn Any + Send + Sync + 'static> =
            unsafe { std::mem::transmute(self.options.get(&TypeId::of::<S>()).cloned()?) };

        match Arc::downcast(any) {
            Ok(v) => Some(v),
            #[cfg(feature = "type_name_of_val")]
            Err(o) => {
                panic!(
                    "failed to downcast {original} into {expected}",
                    original = std::any::type_name_of_val(&o),
                    expected = type_name::<S>(),
                )
            }
            #[cfg(not(feature = "type_name_of_val"))]
            Err(o) => {
                panic!(
                    "failed to downcast {original:?} into {expected}",
                    original = o.type_id(),
                    expected = type_name::<S>(),
                )
            }
        }
    }
}

pub trait Setting: 'static + Any + Send + Sync + fmt::Debug {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Timeout(pub Duration);
#[derive(Debug, Clone, Copy)]
pub struct StackSize(pub usize);

impl Default for Timeout {
    fn default() -> Self {
        Self(Duration::from_millis(2_345))
    }
}
impl Setting for Timeout {}

impl Default for StackSize {
    fn default() -> Self {
        Self(10 * 1_024 * 1_024)
    }
}
impl Setting for StackSize {}
