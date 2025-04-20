// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0
pub trait StorageIterator {
    type KeyType<'a>: PartialEq + Eq + PartialOrd + Ord
    where
        Self: 'a;

    fn value(&self) -> &[u8];
    fn key(&self) -> Self::KeyType<'_>;
    fn is_valid(&self) -> bool;
    fn next(&mut self) -> anyhow::Result<()>;
    fn num_active_iterators(&self) -> usize {
        1
    }
}
