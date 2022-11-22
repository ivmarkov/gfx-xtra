use core::cmp::{max, min};
use core::convert::Infallible;
use core::marker::PhantomData;

use embedded_graphics::prelude::{
    Dimensions, DrawTarget, IntoStorage, OriginDimensions, PixelColor, Point, RawData, Size,
};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

pub struct PackedFramebuffer<'a, COLOR> {
    buf: &'a mut [u8],
    width: usize,
    height: usize,
    _color: PhantomData<COLOR>,
}

impl<'a, COLOR> PackedFramebuffer<'a, COLOR>
where
    COLOR: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    const BITS_PER_PIXEL: usize = Self::bits_per_pixel();
    const PIXEL_MASK: u8 = ((1 << Self::BITS_PER_PIXEL) - 1) as u8;
    const PIXELS_PER_BYTE: usize = 8 / Self::BITS_PER_PIXEL;
    const PIXELS_PER_BYTE_SHIFT: usize = if Self::BITS_PER_PIXEL == 8 {
        0
    } else {
        Self::BITS_PER_PIXEL
    };

    pub fn new(buf: &'a mut [u8], width: usize, height: usize) -> Self {
        Self {
            buf,
            width,
            height,
            _color: PhantomData,
        }
    }

    pub const fn buffer_size(display_size: Size) -> usize {
        display_size.width as usize * display_size.height as usize / (8 / Self::bits_per_pixel())
    }

    const fn bits_per_pixel() -> usize {
        if COLOR::Raw::BITS_PER_PIXEL > 4 {
            8
        } else if COLOR::Raw::BITS_PER_PIXEL > 2 {
            4
        } else if COLOR::Raw::BITS_PER_PIXEL > 1 {
            2
        } else {
            1
        }
    }

    pub fn apply<D>(&mut self, new: &Self, to: &mut D) -> Result<usize, D::Error>
    where
        D: DrawTarget<Color = COLOR>,
    {
        let width = self.width();
        let height = self.height();

        let mut changes = 0_usize;

        let pixels = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let bytes_offset = self.y_offset(y as usize) + Self::x_offset(x as usize);
                let bits_offset = Self::x_bits_offset(x as usize);

                let color = new.get(bytes_offset, bits_offset);
                if self.get(bytes_offset, bits_offset) != color {
                    self.set(bytes_offset, bits_offset, color);

                    changes += 1;

                    Some(Pixel(Point::new(x as _, y as _), color))
                } else {
                    None
                }
            });

        to.draw_iter(pixels)?;

        #[cfg(feature = "log")]
        ::log::trace!(
            "Display updated ({}/{} changed pixels)",
            changes,
            width * height
        );

        Ok(changes)
    }

    fn offsets(&self, area: Rectangle) -> impl Iterator<Item = (usize, usize)> {
        let dimensions = self.bounding_box();
        let bottom_right = dimensions.bottom_right().unwrap_or(dimensions.top_left);

        let x = min(max(area.top_left.x, 0), bottom_right.x) as usize;
        let y = min(max(area.top_left.y, 0), bottom_right.y) as usize;

        let xend = min(
            max(area.top_left.x + area.size.width as i32, 0),
            bottom_right.x,
        ) as usize;
        let yend = min(
            max(area.top_left.y + area.size.height as i32, 0),
            bottom_right.y,
        ) as usize;

        (self.y_offset(y)..self.y_offset(yend))
            .step_by(self.bytes_per_row())
            .flat_map(move |y_offset| {
                (x..xend).map(move |x| (y_offset + Self::x_offset(x), Self::x_bits_offset(x)))
            })
    }

    #[inline(always)]
    fn width(&self) -> usize {
        self.width
    }

    #[inline(always)]
    fn height(&self) -> usize {
        self.height
    }

    #[inline(always)]
    fn to_bits(color: COLOR) -> u8 {
        color.into_storage()
    }

    #[inline(always)]
    fn from_bits(bits: u8) -> COLOR {
        bits.into()
    }

    #[inline(always)]
    fn y_offset(&self, y: usize) -> usize {
        y * self.bytes_per_row()
    }

    #[inline(always)]
    fn x_offset(x: usize) -> usize {
        x / Self::PIXELS_PER_BYTE
    }

    #[inline(always)]
    fn x_bits_offset(x: usize) -> usize {
        Self::PIXELS_PER_BYTE_SHIFT * (x % Self::PIXELS_PER_BYTE)
    }

    #[inline(always)]
    fn bytes_per_row(&self) -> usize {
        self.width() / Self::PIXELS_PER_BYTE
    }

    #[inline(always)]
    fn get(&self, byte_offset: usize, bits_offset: usize) -> COLOR {
        Self::from_bits((self.buf[byte_offset] >> bits_offset) & Self::PIXEL_MASK)
    }

    #[inline(always)]
    fn set(&mut self, byte_offset: usize, bits_offset: usize, color: COLOR) {
        let byte = &mut self.buf[byte_offset];
        *byte &= !(Self::PIXEL_MASK << bits_offset);
        *byte |= Self::to_bits(color) << bits_offset;
    }
}

impl<'a, COLOR> OriginDimensions for PackedFramebuffer<'a, COLOR>
where
    COLOR: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    fn size(&self) -> Size {
        Size::new(self.width() as u32, self.height() as u32)
    }
}

impl<'a, COLOR> DrawTarget for PackedFramebuffer<'a, COLOR>
where
    COLOR: PixelColor + IntoStorage<Storage = u8> + From<u8>,
{
    type Error = Infallible;

    type Color = COLOR;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            if pixel.0.x >= 0
                && pixel.0.x < self.width() as _
                && pixel.0.y >= 0
                && pixel.0.y < self.height() as _
            {
                self.set(
                    self.y_offset(pixel.0.y as usize) + Self::x_offset(pixel.0.x as usize),
                    Self::x_bits_offset(pixel.0.x as usize),
                    pixel.1,
                );
            }
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let mut colors = colors.into_iter();

        for (byte_offset, bits_offset) in self.offsets(*area) {
            if let Some(color) = colors.next() {
                self.set(byte_offset, bits_offset, color);
            }
        }

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        for (byte_offset, bits_offset) in self.offsets(*area) {
            self.set(byte_offset, bits_offset, color);
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        if Self::to_bits(color) == 0 {
            for byte in self.buf.iter_mut() {
                *byte = 0;
            }
        } else {
            for (byte_offset, bits_offset) in self.offsets(self.bounding_box()) {
                self.set(byte_offset, bits_offset, color);
            }
        }

        Ok(())
    }
}
