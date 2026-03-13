// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex)
// Original code from CocoIndex is copyrighted by CocoIndex
// SPDX-FileCopyrightText: 2025-2026 CocoIndex (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for Recoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the Recoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RefList<'a, T> {
    #[default]
    Nil,

    Cons(T, &'a RefList<'a, T>),
}

impl<'a, T> RefList<'a, T> {
    pub fn prepend(&'a self, head: T) -> Self {
        Self::Cons(head, self)
    }

    pub fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        self
    }

    pub fn head(&'a self) -> Option<&'a T> {
        match self {
            RefList::Nil => None,
            RefList::Cons(head, _) => Some(head),
        }
    }

    pub fn headn(&'a self, n: usize) -> Option<&'a T> {
        match self {
            RefList::Nil => None,
            RefList::Cons(head, tail) => {
                if n == 0 {
                    Some(head)
                } else {
                    tail.headn(n - 1)
                }
            }
        }
    }

    pub fn tail(&'a self) -> Option<&'a RefList<'a, T>> {
        match self {
            RefList::Nil => None,
            RefList::Cons(_, tail) => Some(tail),
        }
    }

    pub fn tailn(&'a self, n: usize) -> Option<&'a RefList<'a, T>> {
        if n == 0 {
            Some(self)
        } else {
            match self {
                RefList::Nil => None,
                RefList::Cons(_, tail) => tail.tailn(n - 1),
            }
        }
    }
}

impl<'a, T> Iterator for &'a RefList<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = *self;
        match current {
            RefList::Nil => None,
            RefList::Cons(head, tail) => {
                *self = *tail;
                Some(head)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headn() {
        let nil: RefList<'_, i32> = RefList::Nil;
        assert_eq!(nil.headn(0), None);
        assert_eq!(nil.headn(1), None);

        let n3 = nil.prepend(3);
        let n2 = n3.prepend(2);
        let n1 = n2.prepend(1);

        // List is 1 -> 2 -> 3 -> Nil
        assert_eq!(n1.headn(0), Some(&1));
        assert_eq!(n1.headn(1), Some(&2));
        assert_eq!(n1.headn(2), Some(&3));
        assert_eq!(n1.headn(3), None);
        assert_eq!(n1.headn(10), None);
    }
}
