use crate::KeyName;

/// Small HashMap-like linear storage intended for small collections
/// where hashing overhead might be slightly annoying.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct KeyValueStorage<V> {
    storage: Vec<KeyValuePair<V>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct KeyValuePair<V> {
    pub key: KeyName,
    pub value: V,
}

impl<V> KeyValueStorage<V> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a KeyName, &'a V)> {
        self.storage.iter().map(|pair| (&pair.key, &pair.value))
    }

    pub fn add(&mut self, pair: KeyValuePair<V>) -> bool {
        if let Some(existing_pair) = self.storage.iter_mut().find(|item| item.key == pair.key) {
            existing_pair.value = pair.value;
            false
        } else {
            self.storage.push(pair);
            true
        }
    }

    pub fn put(&mut self, key: KeyName, value: V) -> bool {
        self.add(KeyValuePair { key, value })
    }

    pub fn get<S>(&self, key: S) -> Option<&V>
    where
        S: AsRef<str>,
    {
        self.storage
            .iter()
            .find_map(|item| (item.key.as_str() == key.as_ref()).then_some(&item.value))
    }
}

impl<V> FromIterator<(KeyName, V)> for KeyValueStorage<V> {
    fn from_iter<T: IntoIterator<Item = (KeyName, V)>>(iter: T) -> Self {
        Self {
            storage: iter
                .into_iter()
                .map(|(key, value)| KeyValuePair { key, value })
                .collect(),
        }
    }
}
