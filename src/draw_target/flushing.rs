use embedded_graphics::prelude::{Dimensions, DrawTarget};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

use super::Flushable;

pub struct Flushing<'a, T, F> {
    parent: &'a mut T,
    flusher: F,
}

impl<'a, T, F> Flushing<'a, T, F> {
    pub(crate) fn new(parent: &'a mut T, flusher: F) -> Self {
        Self { parent, flusher }
    }
}

impl<'a, T> Flushing<'a, T, fn(&mut T) -> Result<(), T::Error>>
where
    T: DrawTarget,
{
    pub(crate) fn noop(parent: &'a mut T) -> Self {
        Self::new(parent, |_| Ok(()))
    }
}

impl<'a, T, F> Flushable for Flushing<'a, T, F>
where
    T: DrawTarget,
    F: FnMut(&mut T) -> Result<(), T::Error>,
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        let Self {
            parent: target,
            flusher,
        } = self;

        (flusher)(target)
    }
}

impl<'a, T, F> DrawTarget for Flushing<'a, T, F>
where
    T: DrawTarget,
{
    type Error = T::Error;
    type Color = T::Color;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.parent.draw_iter(pixels)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.parent.fill_contiguous(area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.parent.fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.parent.clear(color)
    }
}

impl<'a, T, F> Dimensions for Flushing<'a, T, F>
where
    T: Dimensions,
{
    fn bounding_box(&self) -> Rectangle {
        self.parent.bounding_box()
    }
}
