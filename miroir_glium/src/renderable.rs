use super::*;

/// A trait encompassing a shape that can be rendered
///
/// [`Mirror`]s implementing [`OpenGLRenderable`] return objects for this trait enabling them to be rendered
/// on-screen in simulations.
pub trait RenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource;
    fn indices(&self) -> gl::index::IndicesSource;
}

/// glium_shapes 3D convenience blanket impl
impl RenderData for glium_shapes::sphere::Sphere {
    fn vertices(&self) -> gl::vertex::VerticesSource<'_> {
        self.into()
    }

    fn indices(&self) -> gl::index::IndicesSource<'_> {
        self.into()
    }
}

/// A wrapper around a `Vec<T>` that only allows pushing/appending/extending etc...
pub struct List<T>(pub(crate) Vec<T>);

/// Most of these methods forward their implementation to the inner [`Vec`].
/// Check the relevant documentation when needed.
impl<T> List<T> {
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    #[inline]
    pub fn push(&mut self, v: T) {
        self.0.push(v);
    }

    #[inline]
    pub fn append(&mut self, vec: &mut Vec<T>) {
        self.0.append(vec);
    }

    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.0.extend_from_slice(slice);
    }
}

impl<T> Extend<T> for List<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

#[impl_trait_for_tuples::impl_for_tuples(16)]
pub trait OpenGLRenderable {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>);
}

impl<T: OpenGLRenderable> OpenGLRenderable for [T] {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.iter()
            .for_each(|a| a.append_render_data(display, list));
    }
}

impl<const N: usize, T: OpenGLRenderable> OpenGLRenderable for [T; N] {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.as_slice().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Box<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.as_ref().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Arc<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.as_ref().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Rc<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.as_ref().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable> OpenGLRenderable for Vec<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.as_slice().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for &T {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        (*self).append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for &mut T {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        (*self as &T).append_render_data(display, list);
    }
}
