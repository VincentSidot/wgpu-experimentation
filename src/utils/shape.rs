macro_rules! shape{
    (@step $idx:expr, $name:ident) => {

        #[allow(non_snake_case)]
        let $name = $idx;
    };
    (@step $idx:expr, $name:ident, $($tail_name:ident),*) => {

        #[allow(non_snake_case)]
        let $name = $idx;
        shape!(@step $idx + 1, $($tail_name),*);
    };
    (
        $($name:ident => $pos:expr, $color: expr),*;
        $($($point:ident) *),*,
    ) => {
        {
            // Create one variable for each point
            // Value of the variable is the index of the point
            shape!(@step 0u16, $($name),*);


            (
                [
                    // Vertex
                    $(crate::graphics::Vertex::new($pos, $color),)*
                ], [
                    // Indices
                    $($($point),*),*
                ]
            )
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
