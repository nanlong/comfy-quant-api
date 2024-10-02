use std::{any::Any, collections::HashMap, fmt};

type AnyMap = HashMap<usize, Box<dyn AnyClone + Send + Sync>>;

#[derive(Clone, Default)]
pub struct Slots {
    data: Option<Box<AnyMap>>,
}

#[allow(unused)]
impl Slots {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set<T>(&mut self, key: usize, val: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.data
            .get_or_insert_with(Box::default)
            .insert(key, Box::new(val))
            .and_then(|boxed| boxed.into_any().downcast().ok().map(|boxed| *boxed))
    }

    pub fn get<T>(&self, key: usize) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.data
            .as_ref()
            .and_then(|map| map.get(&key))
            .and_then(|boxed| (**boxed).as_any().downcast_ref())
    }

    pub fn get_mut<T>(&mut self, key: usize) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.data
            .as_mut()
            .and_then(|map| map.get_mut(&key))
            .and_then(|boxed| (**boxed).as_any_mut().downcast_mut())
    }
}

impl fmt::Debug for Slots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Slots").finish()
    }
}

#[allow(unused)]
pub(crate) trait AnyClone: Any {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Clone + Send + Sync + 'static> AnyClone for T {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Clone for Box<dyn AnyClone + Send + Sync> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slots() {
        #[derive(Clone, Debug, PartialEq)]
        struct MyType(i32);

        let mut slots = Slots::new();

        slots.set(0, 5i32);
        slots.set(1, MyType(10));

        assert_eq!(slots.get(0), Some(&5i32));
        assert_eq!(slots.get_mut(0), Some(&mut 5i32));
    }
}
