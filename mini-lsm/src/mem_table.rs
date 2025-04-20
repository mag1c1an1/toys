#![allow(dead_code)]

use std::{
    ops::Bound,
    path::Path,
    sync::{Arc, atomic::AtomicUsize},
};

use bytes::Bytes;
use crossbeam_skiplist::{
    SkipMap,
    map::{Entry, Range},
};
use ouroboros::self_referencing;

use crate::{
    iterators::StorageIterator,
    key::{KeyBytes, KeySlice},
    wal::Wal,
};

pub struct MemTable {
    pub(crate) map: Arc<SkipMap<KeyBytes, Bytes>>,
    wal: Option<Wal>,
    id: usize,
    approximate_size: Arc<AtomicUsize>,
}

impl MemTable {
    pub fn create(id: usize) -> Self {
        todo!()
    }
    pub fn create_with_wal(id: usize, path: impl AsRef<Path>) -> Self {
        todo!()
    }
}

type SkipMapRangeIter<'a> =
    Range<'a, KeyBytes, (Bound<KeyBytes>, Bound<KeyBytes>), KeyBytes, Bytes>;

#[self_referencing]
pub struct MemTableIterator {
    map: Arc<SkipMap<KeyBytes, Bytes>>,
    #[borrows(map)]
    #[not_covariant]
    iter: SkipMapRangeIter<'this>,
    item: (KeyBytes, Bytes),
}
impl MemTableIterator {
    fn entry_to_item(
        entry: Option<Entry<'_, KeyBytes, Bytes>>,
    ) -> (KeyBytes, Bytes) {
        entry
            .map(|x| (x.key().clone(), x.value().clone()))
            .unwrap_or_else(|| (KeyBytes::new(), Bytes::new()))
    }
}

impl StorageIterator for MemTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    fn value(&self) -> &[u8] {
        &self.borrow_item().1[..]
    }

    fn key(&self) -> Self::KeyType<'_> {
        self.borrow_item().0.as_key_slice()
    }

    fn is_valid(&self) -> bool {
        !self.borrow_item().0.is_empty()
    }

    fn next(&mut self) -> anyhow::Result<()> {
        let entry = self
            .with_iter_mut(|iter| MemTableIterator::entry_to_item(iter.next()));
        self.with_mut(|x| *x.item = entry);
        Ok(())
    }
}
