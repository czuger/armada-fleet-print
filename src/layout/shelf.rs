//! Next-fit shelf packing: rectangles go left to right on a line, a new line starts below the
//! tallest rectangle of the previous one, and a new page starts when a line no longer fits.

/// Packing state over successive pages of the same size.
pub(crate) struct Shelf {
    width: f32,
    height: f32,
    column_gap: f32,
    row_gap: f32,
    /// Left edge of the next rectangle on the current line.
    x: f32,
    /// Top of the current line, from the bottom of the area.
    top: f32,
    /// Height of the tallest rectangle on the current line.
    row_height: f32,
}

/// Where a rectangle was placed.
#[derive(Debug, PartialEq)]
pub(crate) struct Slot {
    /// Left edge, from the left of the area.
    pub left: f32,
    /// Top edge, from the bottom of the area.
    pub top: f32,
    /// Whether the rectangle starts a new page.
    pub new_page: bool,
}

impl Shelf {
    /// Starts packing at the top-left corner of a `width` × `height` area.
    pub(crate) fn new(width: f32, height: f32, column_gap: f32, row_gap: f32) -> Self {
        Self {
            width,
            height,
            column_gap,
            row_gap,
            x: 0.0,
            top: height,
            row_height: 0.0,
        }
    }

    /// Reserves a `width` × `height` rectangle and returns its top-left corner.
    ///
    /// A rectangle too large for an empty page is still placed at its top-left corner.
    pub(crate) fn fit(&mut self, width: f32, height: f32) -> Slot {
        if self.x > 0.0 && self.x + width > self.width {
            self.top -= self.row_height + self.row_gap;
            self.x = 0.0;
            self.row_height = 0.0;
        }
        let page_is_empty = self.x == 0.0 && self.top == self.height;
        let new_page = height > self.top && !page_is_empty;
        if new_page {
            self.break_page();
        }
        let left = self.x;
        self.x += width + self.column_gap;
        self.row_height = self.row_height.max(height);
        Slot {
            left,
            top: self.top,
            new_page,
        }
    }

    /// Moves to the top-left corner of a new page.
    pub(crate) fn break_page(&mut self) {
        self.x = 0.0;
        self.top = self.height;
        self.row_height = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(left: f32, top: f32, new_page: bool) -> Slot {
        Slot {
            left,
            top,
            new_page,
        }
    }

    #[test]
    fn breaks_line_when_width_is_exceeded() {
        let mut shelf = Shelf::new(100.0, 100.0, 5.0, 10.0);
        assert_eq!(shelf.fit(40.0, 20.0), slot(0.0, 100.0, false));
        assert_eq!(shelf.fit(40.0, 30.0), slot(45.0, 100.0, false));
        assert_eq!(shelf.fit(40.0, 20.0), slot(0.0, 60.0, false));
    }

    #[test]
    fn breaks_page_when_height_is_exceeded() {
        let mut shelf = Shelf::new(100.0, 100.0, 5.0, 10.0);
        assert_eq!(shelf.fit(80.0, 50.0), slot(0.0, 100.0, false));
        assert_eq!(shelf.fit(80.0, 50.0), slot(0.0, 100.0, true));
    }

    #[test]
    fn breaks_page_when_a_taller_item_does_not_fit_on_the_line() {
        let mut shelf = Shelf::new(100.0, 100.0, 5.0, 10.0);
        shelf.fit(40.0, 50.0);
        shelf.fit(40.0, 30.0);
        assert_eq!(shelf.fit(40.0, 45.0), slot(0.0, 100.0, true));
    }

    #[test]
    fn places_oversized_item_on_empty_page() {
        let mut shelf = Shelf::new(100.0, 100.0, 5.0, 10.0);
        assert_eq!(shelf.fit(150.0, 150.0), slot(0.0, 100.0, false));
        assert_eq!(shelf.fit(10.0, 10.0), slot(0.0, 100.0, true));
    }
}
