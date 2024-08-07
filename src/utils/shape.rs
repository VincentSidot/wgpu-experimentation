macro_rules! shape{
    (@step $idx:expr, $vec: ident, $color: expr, $name:ident, $pos:expr) => {

        #[allow(non_snake_case)]
        let $name = $idx;

        $vec.push(
            crate::graphics::Vertex::new($pos, $color)
        );
    };
    (@step $idx:expr, $vec:ident, $color:expr, $name:ident, $pos: expr, $($tail_color: expr, $tail_name:ident, $tail_pos: expr),*) => {

        #[allow(non_snake_case)]
        let $name = $idx;
        $vec.push(crate::graphics::Vertex::new($pos, $color));
        shape!(@step $idx + 1, $vec, $($tail_color, $tail_name, $tail_pos),*);
    };
    (
        $($name:ident => $pos:expr, $color: expr),*;
        // $($pointA:ident $pointB:ident $pointC:ident),*,
        $($($point:ident) *),*,
    ) => {
        {

            let mut vertex = Vec::new();
            let mut indices = Vec::new();


            shape!(@step 0u16, vertex, $($color, $name, $pos),*);

            $(
                $(
                    indices.push($point);
                )*
            )*


            (vertex, indices)
        }
    };
    (
        $color: expr;
        $($name:ident => $pos:expr),*;
        // $($pointA:ident $pointB:ident $pointC:ident),*,
        $($($point:ident) *),*,
    ) => {
        shape!(
            $($name => $pos, $color),*;
            $($($point) *),*,
        )
    };
}

pub(crate) use shape;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_shape() {
        let (vertex, indices) = shape!(
            [1.0, 0.0, 0.0]; // Red
            A => [0.0, 0.0, 0.0],
            B => [1.0, 0.0, 0.0],
            C => [1.0, 1.0, 0.0];
            A B C,
        );

        assert_eq!(vertex.len(), 3);
        assert_eq!(indices.len(), 3);
    }
}
