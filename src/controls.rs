use sfml::graphics::{    Color, Font, PrimitiveType, RectangleShape, RenderStates, RenderTarget, RenderWindow, Shape,
    Text, Transformable, Vertex,Rect,
};
use sfml::system::Vector2f;

struct Control{
    rect: Rect
}

impl Control{
    fn new(rect: Rect<usize>)-> Control{
Control{rect}
    }
}

