use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UuidMap<T> {
    items: HashMap<Uuid, T>,
    order: Vec<Uuid>,
}

impl<T> UuidMap<T> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn add(&mut self, item: T) -> Uuid {
        let uuid = Uuid::new_v4();
        self.items.insert(uuid, item);
        self.order.push(uuid);
        uuid
    }

    pub fn remove(&mut self, id: Uuid) -> Option<T> {
        self.order.retain(|x| *x != id);
        self.items.remove(&id)
    }

    pub fn uuids(&self) -> impl DoubleEndedIterator<Item = Uuid> {
        self.order.iter().copied()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Uuid, &T)> {
        self.uuids().map(|uuid| (uuid, &self[uuid]))
    }

    pub fn values(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.uuids().map(|uuid| &self.items[&uuid])
    }

    pub fn values_mut_unordered(&mut self) -> impl ExactSizeIterator<Item = &mut T> {
        self.items.values_mut()
    }

    pub fn into_iter(mut self) -> impl Iterator<Item = (Uuid, T)> {
        self.order
            .into_iter()
            .filter_map(move |id| Some((id, self.items.remove(&id)?)))
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn get_uuid(&self, index: usize) -> Option<Uuid> {
        self.order.get(index).copied()
    }
}

impl<T> Default for UuidMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<Uuid> for UuidMap<T> {
    type Output = T;

    fn index(&self, index: Uuid) -> &Self::Output {
        &self.items[&index]
    }
}

impl<T> IndexMut<Uuid> for UuidMap<T> {
    #[track_caller]
    fn index_mut(&mut self, index: Uuid) -> &mut Self::Output {
        self.items.get_mut(&index).unwrap()
    }
}

impl<T> Index<usize> for UuidMap<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self[self.order[index]]
    }
}

impl<T> IndexMut<usize> for UuidMap<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let id = self.order[index];
        &mut self[id]
    }
}

impl<T> FromIterator<(Uuid, T)> for UuidMap<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Uuid, T)>,
    {
        let mut this = Self::new();

        for (id, item) in iter {
            this.items.insert(id, item);
            this.order.push(id);
        }

        this
    }
}
