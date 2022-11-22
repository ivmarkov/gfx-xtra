use embedded_graphics::prelude::{DrawTarget, IntoStorage, OriginDimensions, PixelColor, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

use super::{Flushable, PackedFramebuffer};

pub struct Buffered<'a, T>
where
    T: DrawTarget,
{
    current: PackedFramebuffer<'a, T::Color>,
    reference: PackedFramebuffer<'a, T::Color>,
    target: T,
}

pub const fn buffer_size<C>(display_size: Size) -> usize
where
    C: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    PackedFramebuffer::<C>::buffer_size(display_size)
}

impl<'a, T> Buffered<'a, T>
where
    T: DrawTarget,
    T::Color: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    pub(crate) fn new(draw_buf: &'a mut [u8], reference_buf: &'a mut [u8], display: T) -> Self {
        let bbox = display.bounding_box();

        Self {
            current: PackedFramebuffer::<T::Color>::new(
                draw_buf,
                bbox.size.width as _,
                bbox.size.height as _,
            ),
            reference: PackedFramebuffer::<T::Color>::new(
                reference_buf,
                bbox.size.width as _,
                bbox.size.height as _,
            ),
            target: display,
        }
    }
}

impl<'a, T> OriginDimensions for Buffered<'a, T>
where
    T: DrawTarget,
    T::Color: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    fn size(&self) -> Size {
        self.current.size()
    }
}

impl<'a, T> DrawTarget for Buffered<'a, T>
where
    T: DrawTarget,
    T::Color: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    type Error = T::Error;

    type Color = T::Color;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.current.draw_iter(pixels).unwrap();

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.current.fill_contiguous(area, colors).unwrap();

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.current.fill_solid(area, color).unwrap();

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.current.clear(color).unwrap();

        Ok(())
    }
}

impl<'a, T> Flushable for Buffered<'a, T>
where
    T: Flushable,
    T::Color: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.reference.apply(&self.current, &mut self.target)?;

        self.target.flush()
    }
}
