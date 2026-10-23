use super::*;

impl<S: miroir::na::RealField + AsPrimitive<i16>> KandinskyRenderable
    for miroir_shapes::Sphere<S, 2>
{
    fn draw(&self, color: kandinsky::Color) {
        kandinsky::draw_circle(
            self.center.to_point(),
            self.radius().as_().unsigned_abs(),
            color,
        );
    }
}

impl<S: miroir::na::RealField + AsPrimitive<i16>> KandinskyRenderable
    for miroir_shapes::LineSegment<S>
{
    fn draw(&self, color: kandinsky::Color) {
        let [start, end] = self.vertices();
        kandinsky::draw_line(start.to_point(), end.to_point(), color);
    }
}
