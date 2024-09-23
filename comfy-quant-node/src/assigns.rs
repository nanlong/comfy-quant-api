use std::{any::Any, collections::HashMap, fmt};

type AnyMap = HashMap<String, Box<dyn AnyClone + Send + Sync>>;

#[derive(Clone, Default)]
pub struct Assigns {
    map: Option<Box<AnyMap>>,
}

impl Assigns {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    ///
    /// # Example
    ///
    /// ```
    /// # use axum_ws::Assigns;
    /// let mut assigns = Assigns::new();
    /// assert!(assigns.insert("my_key", 5i32).is_none());
    /// assert_eq!(assigns.insert("my_key", 9i32), Some(5i32));
    /// ```
    pub fn insert<T>(&mut self, key: impl Into<String>, val: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.map
            .get_or_insert_with(Box::default)
            .insert(key.into(), Box::new(val))
            .and_then(|boxed| boxed.into_any().downcast().ok().map(|boxed| *boxed))
    }

    ///
    /// # Example
    ///
    /// ```
    /// # use axum_ws::Assigns;
    /// let mut assigns = Assigns::new();
    /// assert!(assigns.get::<i32>("my_key").is_none());
    /// assigns.insert("my_key", 5i32);
    ///
    /// assert_eq!(assigns.get::<i32>("my_key"), Some(&5i32));
    /// ```
    pub fn get<T>(&self, key: impl AsRef<str>) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .as_ref()
            .and_then(|map| map.get(key.as_ref()))
            .and_then(|boxed| (**boxed).as_any().downcast_ref())
    }

    pub fn get_mut<T>(&mut self, key: impl AsRef<str>) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .as_mut()
            .and_then(|map| map.get_mut(key.as_ref()))
            .and_then(|boxed| (**boxed).as_any_mut().downcast_mut())
    }

    pub fn get_or_insert<T>(&mut self, key: impl Into<String>, value: T) -> &mut T
    where
        T: Clone + Send + Sync + 'static,
    {
        self.get_or_insert_with(key, || value)
    }

    pub fn get_or_insert_with<T, F>(&mut self, key: impl Into<String>, f: F) -> &mut T
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let out = self
            .map
            .get_or_insert_with(Box::default)
            .entry(key.into())
            .or_insert_with(|| Box::new(f()));

        (**out).as_any_mut().downcast_mut().unwrap()
    }

    pub fn get_or_insert_default<T>(&mut self, key: impl Into<String>) -> &mut T
    where
        T: Default + Clone + Send + Sync + 'static,
    {
        self.get_or_insert_with(key, T::default)
    }

    pub fn remove<T>(&mut self, key: impl AsRef<str>) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .as_mut()
            .and_then(|map| map.remove(key.as_ref()))
            .and_then(|boxed| boxed.into_any().downcast().ok().map(|boxed| *boxed))
    }

    #[inline]
    pub fn clear(&mut self) {
        if let Some(ref mut map) = self.map {
            map.clear();
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.as_ref().map_or(true, |map| map.is_empty())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.as_ref().map_or(0, |map| map.len())
    }

    pub fn extend(&mut self, other: Self) {
        if let Some(other) = other.map {
            if let Some(map) = &mut self.map {
                map.extend(*other);
            } else {
                self.map = Some(other);
            }
        }
    }
}

impl fmt::Debug for Assigns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Assigns").finish()
    }
}

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

#[test]
fn test_assigns() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyType(i32);

    let mut assigns = Assigns::new();

    assigns.insert("my_key1", 5i32);
    assigns.insert("my_key2", MyType(10));

    assert_eq!(assigns.get("my_key1"), Some(&5i32));
    assert_eq!(assigns.get_mut("my_key1"), Some(&mut 5i32));

    let assigns2 = assigns.clone();

    assert_eq!(assigns.remove("my_key1"), Some(5i32));
    assert!(assigns.get::<i32>("my_key1").is_none());

    // clone still has it
    assert_eq!(assigns2.get("my_key1"), Some(&5i32));
    assert_eq!(assigns2.get("my_key2"), Some(&MyType(10)));

    assert_eq!(assigns.get::<bool>("my_key3"), None);
    assert_eq!(assigns.get("my_key2"), Some(&MyType(10)));
}
