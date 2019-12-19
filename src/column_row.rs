use std::ops::Range;

pub struct ColumnRowIter<T: Copy + core::cmp::PartialOrd + core::ops::AddAssign + From<u8>> {
    range_row: Range<T>,
    row_pos: T,
    range_column: Range<T>,
    col_pos: T,
}

impl<T> Iterator for ColumnRowIter<T>
where
    T: core::marker::Copy + core::cmp::PartialOrd + core::ops::AddAssign + From<u8>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<(T, T)> {
        while self.range_column.contains(&self.col_pos) {
            if self.range_row.contains(&self.row_pos) {
                let row_pos = self.row_pos;
                self.row_pos += 1.into();
                return Some((self.col_pos, row_pos));
            }
            self.col_pos += 1.into();
            self.row_pos = self.range_row.start;
        }
        None
    }
}

pub trait RangeExt<T: Copy + core::cmp::PartialOrd + core::ops::AddAssign + From<u8>> {
    fn rows(self, r: Range<T>) -> ColumnRowIter<T>;
}

impl<T: Copy> RangeExt<T> for Range<T>
where
    T: core::marker::Copy + core::cmp::PartialOrd + core::ops::AddAssign + From<u8>,
{
    fn rows(self, rows: Range<T>) -> ColumnRowIter<T> {
        ColumnRowIter {
            range_column: self.clone(),
            col_pos: self.start,
            range_row: rows.clone(),
            row_pos: rows.start,
        }
    }
}
