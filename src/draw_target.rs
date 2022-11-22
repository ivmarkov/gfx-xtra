use core::marker::PhantomData;

use embedded_graphics::{
    prelude::{DrawTarget, IntoStorage, PixelColor, Point, Size},
    primitives::Rectangle,
};

pub use buffered::*;
pub use fb::*;
pub use flushing::*;
pub use owned::*;
pub use rotated::*;
pub use scaled::*;

mod buffered;
mod fb;
mod flushing;
mod owned;
mod rotated;
mod scaled;

pub trait Flushable: DrawTarget {
    fn flush(&mut self) -> Result<(), Self::Error>;
}

pub trait DrawTargetExt2: DrawTarget + Sized {
    fn rotated(&mut self, angle: RotateAngle) -> Rotated<'_, Self>;

    fn scaled(&mut self, size: Size) -> Scaled<'_, Self>;

    fn flushing<F: FnMut(&mut Self) -> Result<(), Self::Error>>(
        &mut self,
        flusher: F,
    ) -> Flushing<'_, Self, F>;

    #[allow(clippy::type_complexity)]
    fn noop_flushing(&mut self) -> Flushing<'_, Self, fn(&mut Self) -> Result<(), Self::Error>>;
}

impl<T> DrawTargetExt2 for T
where
    T: DrawTarget,
{
    fn rotated(&mut self, angle: RotateAngle) -> Rotated<'_, Self> {
        Rotated::new(self, angle)
    }

    fn scaled(&mut self, size: Size) -> Scaled<'_, Self> {
        Scaled::new(self, size)
    }

    fn flushing<F: FnMut(&mut Self) -> Result<(), Self::Error>>(
        &mut self,
        flusher: F,
    ) -> Flushing<'_, Self, F> {
        Flushing::new(self, flusher)
    }

    fn noop_flushing(&mut self) -> Flushing<'_, Self, fn(&mut Self) -> Result<(), Self::Error>> {
        Flushing::noop(self)
    }
}

pub trait OwnedDrawTargetExt: DrawTarget + Sized {
    fn owned_translated(self, offset: Point) -> Owned<TranslatedT<Self>>;

    fn owned_cropped(self, area: &Rectangle) -> Owned<CroppedT<Self>>;

    fn owned_clipped(self, area: &Rectangle) -> Owned<ClippedT<Self>>;

    fn owned_color_converted<C>(self) -> Owned<ColorConvertedT<Self, C>>
    where
        C: PixelColor + Into<Self::Color>;

    fn owned_rotated(self, angle: RotateAngle) -> Owned<RotatedT<Self>>;

    fn owned_scaled(self, size: Size) -> Owned<ScaledT<Self>>;

    fn owned_flushing<F: FnMut(&mut Self) -> Result<(), Self::Error> + Send + Clone + 'static>(
        self,
        flusher: F,
    ) -> Owned<FlushingT<Self, F>>
    where
        Self: 'static,
        Self::Error: 'static;

    #[allow(clippy::type_complexity)]
    fn owned_noop_flushing(
        self,
    ) -> Owned<FlushingT<Self, fn(&mut Self) -> Result<(), Self::Error>>>
    where
        Self: 'static,
        Self::Error: 'static;

    fn owned_buffered<'a>(
        self,
        draw_buf: &'a mut [u8],
        reference_buf: &'a mut [u8],
    ) -> Buffered<'a, Self>
    where
        Self::Color: PixelColor + IntoStorage<Storage = u8> + From<u8>;
}

impl<T> OwnedDrawTargetExt for T
where
    T: DrawTarget,
{
    fn owned_translated(self, offset: Point) -> Owned<TranslatedT<Self>> {
        TranslatedT(self, offset).into_owned()
    }

    fn owned_cropped(self, area: &Rectangle) -> Owned<CroppedT<Self>> {
        CroppedT(self, *area).into_owned()
    }

    fn owned_clipped(self, area: &Rectangle) -> Owned<ClippedT<Self>> {
        ClippedT(self, *area).into_owned()
    }

    fn owned_color_converted<C>(self) -> Owned<ColorConvertedT<Self, C>>
    where
        C: PixelColor + Into<Self::Color>,
    {
        ColorConvertedT(self, PhantomData::<C>).into_owned()
    }

    fn owned_rotated(self, angle: RotateAngle) -> Owned<RotatedT<Self>> {
        RotatedT(self, angle).into_owned()
    }

    fn owned_scaled(self, size: Size) -> Owned<ScaledT<Self>> {
        ScaledT(self, size).into_owned()
    }

    fn owned_flushing<F: FnMut(&mut Self) -> Result<(), Self::Error> + Send + Clone + 'static>(
        self,
        flusher: F,
    ) -> Owned<FlushingT<Self, F>>
    where
        Self: 'static,
        Self::Error: 'static,
    {
        FlushingT(self, flusher).into_owned()
    }

    fn owned_noop_flushing(self) -> Owned<FlushingT<Self, fn(&mut Self) -> Result<(), Self::Error>>>
    where
        Self: 'static,
        Self::Error: 'static,
    {
        self.owned_flushing(|_| Ok(()))
    }

    fn owned_buffered<'a>(
        self,
        draw_buf: &'a mut [u8],
        reference_buf: &'a mut [u8],
    ) -> Buffered<'a, Self>
    where
        Self::Color: PixelColor + IntoStorage<Storage = u8> + From<u8>,
    {
        Buffered::new(draw_buf, reference_buf, self)
    }
}
