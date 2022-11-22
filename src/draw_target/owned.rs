use core::marker::PhantomData;

use embedded_graphics::draw_target::{Clipped, ColorConverted, Cropped, DrawTargetExt, Translated};
use embedded_graphics::prelude::{Dimensions, DrawTarget, PixelColor, Point, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

use super::{DrawTargetExt2, Flushable, Flushing, RotateAngle, Rotated, Scaled};

pub trait Transformer {
    type Color: PixelColor;
    type Error;

    type DrawTarget<'a>: DrawTarget<Color = Self::Color, Error = Self::Error>
    where
        Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_>;

    fn into_owned(self) -> Owned<Self>
    where
        Self: Sized,
    {
        Owned::new(self)
    }
}

pub struct TranslatedT<T>(pub(crate) T, pub(crate) Point);

impl<T> Transformer for TranslatedT<T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    type DrawTarget<'a> = Translated<'a, T> where Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_> {
        self.0.translated(self.1)
    }
}

pub struct CroppedT<T>(pub(crate) T, pub(crate) Rectangle);

impl<T> Transformer for CroppedT<T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    type DrawTarget<'a> = Cropped<'a, T> where Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_> {
        self.0.cropped(&self.1)
    }
}

pub struct ClippedT<T>(pub(crate) T, pub(crate) Rectangle);

impl<T> Transformer for ClippedT<T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    type DrawTarget<'a> = Clipped<'a, T> where Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_> {
        self.0.clipped(&self.1)
    }
}

pub struct ColorConvertedT<T, C>(pub(crate) T, pub(crate) PhantomData<C>);

impl<T, C> Transformer for ColorConvertedT<T, C>
where
    T: DrawTarget,
    C: PixelColor + Into<T::Color>,
{
    type Color = C;
    type Error = T::Error;

    type DrawTarget<'a> = ColorConverted<'a, T, C> where Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_> {
        self.0.color_converted()
    }
}

pub struct RotatedT<T>(pub(crate) T, pub(crate) RotateAngle);

impl<T> Transformer for RotatedT<T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    type DrawTarget<'a> = Rotated<'a, T> where Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_> {
        self.0.rotated(self.1)
    }
}

pub struct ScaledT<T>(pub(crate) T, pub(crate) Size);

impl<T> Transformer for ScaledT<T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    type DrawTarget<'a> = Scaled<'a, T> where Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_> {
        self.0.scaled(self.1)
    }
}

pub struct FlushingT<T, F>(pub(crate) T, pub(crate) F);

impl<T, F> Transformer for FlushingT<T, F>
where
    T: DrawTarget + 'static,
    F: FnMut(&mut T) -> Result<(), T::Error> + Send + Clone + 'static,
{
    type Color = T::Color;
    type Error = T::Error;

    type DrawTarget<'a> = Flushing<'a, T, F> where Self: 'a;

    fn transform(&mut self) -> Self::DrawTarget<'_> {
        self.0.flushing(self.1.clone())
    }
}

pub struct Owned<T>(pub(crate) T, pub(crate) Rectangle);

impl<T> Owned<T>
where
    T: Transformer,
{
    fn new(mut transformer: T) -> Self {
        let bbox = transformer.transform().bounding_box();

        Self(transformer, bbox)
    }
}

impl<T> DrawTarget for Owned<T>
where
    T: Transformer,
{
    type Color = T::Color;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0.transform().draw_iter(pixels)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.0.transform().fill_contiguous(area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.0.transform().fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.0.transform().clear(color)
    }
}

impl<T> Dimensions for Owned<T>
where
    T: Transformer,
{
    fn bounding_box(&self) -> Rectangle {
        self.1
    }
}

impl<T> Flushable for Owned<T>
where
    T: Transformer,
    for<'a> T::DrawTarget<'a>: Flushable,
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.transform().flush()
    }
}
