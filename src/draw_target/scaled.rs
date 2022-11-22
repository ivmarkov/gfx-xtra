use embedded_graphics::prelude::{DrawTarget, OriginDimensions, Point, PointsIter, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

pub struct Scaled<'a, T>
where
    T: DrawTarget,
{
    parent: &'a mut T,
    size: Size,
}

impl<'a, T> Scaled<'a, T>
where
    T: DrawTarget,
{
    pub(crate) fn new(parent: &'a mut T, size: Size) -> Self {
        Self { parent, size }
    }

    fn transform(point: Point, size: Size, pdim: &Rectangle) -> Point {
        Point::new(
            pdim.top_left.x + point.x * size.width as i32 / pdim.size.width as i32,
            pdim.top_left.y + point.y * size.height as i32 / pdim.size.height as i32,
        )
    }
}

impl<'a, T> DrawTarget for Scaled<'a, T>
where
    T: DrawTarget,
{
    type Error = T::Error;
    type Color = T::Color;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let pdim = self.parent.bounding_box();
        let size = self.size;

        self.parent.draw_iter(
            pixels
                .into_iter()
                .map(|pixel| Pixel(Self::transform(pixel.0, size, &pdim), pixel.1)),
        )
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let pdim = self.parent.bounding_box();
        let size = self.size;

        self.parent.draw_iter(
            area.points()
                .zip(colors)
                .map(|(pos, color)| Pixel(Self::transform(pos, size, &pdim), color)),
        )
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = Rectangle::new(area.top_left, self.size);

        self.parent.fill_solid(&area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.parent.clear(color)
    }
}

impl<'a, T> OriginDimensions for Scaled<'a, T>
where
    T: DrawTarget,
{
    fn size(&self) -> Size {
        self.size
    }
}
