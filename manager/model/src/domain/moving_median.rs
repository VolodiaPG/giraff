use std::sync::Arc;

use nutype::nutype;
use skiplist::OrderedSkipList;
use uom::si::f64::Time;

#[derive(Debug)]
struct RingOrderedSkipList<T: PartialOrd> {
    ring:        Vec<Arc<T>>,
    window_size: usize,
    cursor:      usize,
    skiplist:    OrderedSkipList<Arc<T>>,
}

impl<T: PartialOrd> RingOrderedSkipList<T> {
    pub fn with_size(window_size: usize) -> Self {
        Self {
            ring: Vec::with_capacity(window_size),
            window_size,
            cursor: 0,
            skiplist: OrderedSkipList::with_capacity(window_size),
        }
    }

    pub fn insert(&mut self, value: Arc<T>) {
        if self.ring.len() == self.window_size {
            if let Some(old) = self.ring.get(self.cursor) {
                self.skiplist.remove(old).unwrap();
            }
            self.ring[self.cursor] = value.clone();
            self.cursor = (self.cursor + 1) % self.window_size;
        } else {
            self.ring.push(value.clone());
        }
        self.skiplist.insert(value);
    }

    pub fn get_ordered(&self, index: usize) -> Option<&Arc<T>> {
        self.skiplist.get(index)
    }

    pub fn len(&self) -> usize { self.ring.len() }
}

#[derive(Debug)]
pub struct MovingMedian {
    window: RingOrderedSkipList<Time>,
}

#[nutype(derive(Debug, Clone), validate(greater_or_equal = 4))]
pub struct MovingMedianSize(usize);

impl MovingMedian {
    pub fn new(size: MovingMedianSize) -> Self {
        MovingMedian {
            window: RingOrderedSkipList::with_size(size.into_inner()),
        }
    }

    pub fn update(&mut self, value: Time) {
        self.window.insert(Arc::new(value))
    }

    fn _get_even(&self, index1: usize, index2: usize) -> Option<Time> {
        let mid1 = self.window.get_ordered(index1);
        let mid2 = self.window.get_ordered(index2);
        match (mid1, mid2) {
            (None, None) => None,
            (None, Some(med)) | (Some(med), None) => Some(*med.as_ref()),
            (Some(mid1), Some(mid2)) => {
                let mid1 = mid1.as_ref();
                let mid2 = mid2.as_ref();
                Some((*mid1 + *mid2) / 2.0)
            }
        }
    }

    fn _get_uneven(&self, index: usize) -> Option<Time> {
        self.window.get_ordered(index).map(|x| *x.as_ref())
    }

    fn _q1(&self) -> Option<Time> {
        let len = self.window.len();
        if len < 4 {
            return None;
        }
        let index1 = len / 4;
        match len % 4 {
            0 | 1 => self._get_even(index1, index1 - 1),
            _ => self._get_uneven(index1),
        }
    }

    fn _q3(&self) -> Option<Time> {
        let len = self.window.len();
        if len < 4 {
            return None;
        }
        let index1 = len * 3 / 4;
        match len % 4 {
            0 | 1 => self._get_even(index1, index1 - 1),
            _ => self._get_uneven(index1),
        }
    }

    pub fn median(&self) -> Option<Time> {
        let len = self.window.len();
        if len < 4 {
            return None;
        }
        let index = len / 2;
        match len % 2 {
            0 => self._get_even(index - 1, index),
            _ => self._get_uneven(index),
        }
    }

    pub fn interquantile_range(&self) -> Option<Time> {
        let q1 = self._q1();
        let q3 = self._q3();
        match (q1, q3) {
            (Some(q1), Some(q3)) => Some(q3 - q1),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use uom::si::f64::Time;
    use uom::si::time::second;
    // Note this useful idiom: importing names from outer (for mod tests)
    // scope.
    use super::*;

    #[test]
    fn test_moving_median() {
        let mut median =
            MovingMedian::new(MovingMedianSize::try_new(10).unwrap());
        median.update(Time::new::<second>(10.0));
        median.update(Time::new::<second>(10.0));
        median.update(Time::new::<second>(0.0));
        median.update(Time::new::<second>(0.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(5.0));
        median.update(Time::new::<second>(5.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(5.0));
        median.update(Time::new::<second>(50.0));
        median.update(Time::new::<second>(45.0));
        median.update(Time::new::<second>(9.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(9.5));
        median.update(Time::new::<second>(10.5));
        assert_eq!(median.median().unwrap(), Time::new::<second>(10.0));
    }

    #[test]
    fn test_moving_median_window() {
        let mut median =
            MovingMedian::new(MovingMedianSize::try_new(5).unwrap());
        median.update(Time::new::<second>(0.0));
        median.update(Time::new::<second>(10.0));
        median.update(Time::new::<second>(5.0));
        median.update(Time::new::<second>(6.0));
        median.update(Time::new::<second>(7.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(6.0));
        median.update(Time::new::<second>(50.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(7.0));
        median.update(Time::new::<second>(5.0));
        median.update(Time::new::<second>(9.0));
        median.update(Time::new::<second>(10.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(9.0));
    }

    #[test]
    fn test_interquantile_range_uneven() {
        let mut median =
            MovingMedian::new(MovingMedianSize::try_new(20).unwrap());
        median.update(Time::new::<second>(6.0));
        median.update(Time::new::<second>(7.0));
        median.update(Time::new::<second>(15.0));
        median.update(Time::new::<second>(36.0));
        median.update(Time::new::<second>(39.0));
        median.update(Time::new::<second>(40.0));
        median.update(Time::new::<second>(41.0));
        median.update(Time::new::<second>(42.0));
        median.update(Time::new::<second>(43.0));
        median.update(Time::new::<second>(47.0));
        median.update(Time::new::<second>(49.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(40.0));
        assert_eq!(median._q1().unwrap(), Time::new::<second>(15.0));
        assert_eq!(median._q3().unwrap(), Time::new::<second>(43.0));
        assert_eq!(
            median.interquantile_range().unwrap(),
            Time::new::<second>(28.0)
        );
    }

    #[test]
    fn test_interquantile_range_even() {
        let mut median =
            MovingMedian::new(MovingMedianSize::try_new(20).unwrap());
        median.update(Time::new::<second>(7.0));
        median.update(Time::new::<second>(15.0));
        median.update(Time::new::<second>(36.0));
        median.update(Time::new::<second>(39.0));
        median.update(Time::new::<second>(40.0));
        median.update(Time::new::<second>(41.0));
        assert_eq!(median.median().unwrap(), Time::new::<second>(37.5));
        assert_eq!(median._q1().unwrap(), Time::new::<second>(15.0));
        assert_eq!(median._q3().unwrap(), Time::new::<second>(40.0));
        assert_eq!(
            median.interquantile_range().unwrap(),
            Time::new::<second>(25.0)
        );
    }
}
