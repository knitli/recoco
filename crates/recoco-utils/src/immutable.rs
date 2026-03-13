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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflist_prepend() {
        let list1: RefList<'_, i32> = RefList::Nil;
        let list2 = list1.prepend(3);
        let list3 = list2.prepend(2);
        let list4 = list3.prepend(1);

        assert_eq!(list4.head(), Some(&1));
        assert_eq!(list4.headn(0), Some(&1));
        assert_eq!(list4.headn(1), Some(&2));
        assert_eq!(list4.headn(2), Some(&3));
        assert_eq!(list4.headn(3), None);

        assert_eq!(list4.tail().unwrap().head(), Some(&2));
        assert_eq!(list4.tailn(0).unwrap().head(), Some(&1));
        assert_eq!(list4.tailn(1).unwrap().head(), Some(&2));
        assert_eq!(list4.tailn(2).unwrap().head(), Some(&3));
        assert!(matches!(list4.tailn(3).unwrap(), RefList::Nil));
        assert_eq!(list4.tailn(4), None);

        let items: Vec<&i32> = list4.iter().collect();
        assert_eq!(items, vec![&1, &2, &3]);
    fn tailn_on_nil() {
        let nil_list: RefList<i32> = RefList::Nil;
        assert_eq!(nil_list.tailn(0), Some(&nil_list));
        assert_eq!(nil_list.tailn(1), None);
    }

    #[test]
    fn tailn_in_bounds() {
        let nil_list: RefList<i32> = RefList::Nil;
        let list1 = RefList::Cons(3, &nil_list);
        let list2 = RefList::Cons(2, &list1);
        let list3 = RefList::Cons(1, &list2);

        assert_eq!(list3.tailn(0), Some(&list3));
        assert_eq!(list3.tailn(1), Some(&list2));
        assert_eq!(list3.tailn(2), Some(&list1));
    }

    #[test]
    fn tailn_exact_length() {
        let nil_list: RefList<i32> = RefList::Nil;
        let list1 = RefList::Cons(3, &nil_list);
        let list2 = RefList::Cons(2, &list1);
        let list3 = RefList::Cons(1, &list2);

        assert_eq!(list3.tailn(3), Some(&nil_list));
    }

    #[test]
    fn tailn_out_of_bounds() {
        let nil_list: RefList<i32> = RefList::Nil;
        let list1 = RefList::Cons(3, &nil_list);
        let list2 = RefList::Cons(2, &list1);
        let list3 = RefList::Cons(1, &list2);

        assert_eq!(list3.tailn(4), None);
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
