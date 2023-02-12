use std::ops::Range;

pub trait Center<T, S> {
    fn center(self) -> Range<S>;
}

impl Center<usize, isize> for Range<usize> {
    fn center(self) -> Range<isize> {
        let center = (self.end as isize + self.start as isize) / 2;

        self.start as isize - center .. self.end as isize - center
    }
}