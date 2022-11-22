use core::cmp::{max, min};

use embedded_graphics::prelude::{DrawTarget, OriginDimensions, Point, PointsIter, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RotateAngle {
    Degrees90,
    Degrees180,
    Degrees270,
}

impl RotateAngle {
    fn transform(&self, point: Point, pdim: &Rectangle) -> Point {
        match self {
            RotateAngle::Degrees90 => Point::new(
                pdim.top_left.x + pdim.size.width as i32 - point.y,
                pdim.top_left.y + point.x,
            ),
            RotateAngle::Degrees180 => Point::new(
                pdim.top_left.x + pdim.size.height as i32 - point.x,
                pdim.top_left.y + pdim.size.width as i32 - point.y,
            ),
            RotateAngle::Degrees270 => Point::new(
                pdim.top_left.x + point.y,
                pdim.top_left.y + pdim.size.height as i32 - point.x,
            ),
        }
    }

    fn transform_size(&self, size: Size) -> Size {
        if *self != RotateAngle::Degrees180 {
            Size::new(size.height, size.width)
        } else {
            size
        }
    }

    fn transform_rect(&self, rect: &Rectangle, pdim: &Rectangle) -> Rectangle {
        let point1 = self.transform(rect.top_left, pdim);
        let point2 = self.transform(rect.top_left + rect.size, pdim);

        let x1 = min(point1.x, point2.x);
        let y1 = min(point1.y, point2.y);

        let x2 = max(point1.x, point2.x);
        let y2 = max(point1.y, point2.y);

        Rectangle::with_corners(Point::new(x1, y1), Point::new(x2, y2))
    }
}

pub struct Rotated<'a, T>
where
    T: DrawTarget,
{
    parent: &'a mut T,
    angle: RotateAngle,
}

impl<'a, T> Rotated<'a, T>
where
    T: DrawTarget,
{
    pub(crate) fn new(parent: &'a mut T, angle: RotateAngle) -> Self {
        Self { parent, angle }
    }
}

impl<'a, T> DrawTarget for Rotated<'a, T>
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
        let angle = self.angle;

        self.parent.draw_iter(
            pixels
                .into_iter()
                .map(|pixel| Pixel(angle.transform(pixel.0, &pdim), pixel.1)),
        )
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let pdim = self.parent.bounding_box();
        let angle = self.angle;

        self.parent.draw_iter(
            area.points()
                .zip(colors)
                .map(|(pos, color)| Pixel(angle.transform(pos, &pdim), color)),
        )
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let pdim = self.parent.bounding_box();
        let angle = self.angle;

        self.parent
            .fill_solid(&angle.transform_rect(area, &pdim), color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.parent.clear(color)
    }
}

impl<'a, T> OriginDimensions for Rotated<'a, T>
where
    T: DrawTarget,
{
    fn size(&self) -> Size {
        let bbox = self.parent.bounding_box();

        self.angle.transform_size(bbox.size)
    }
}
