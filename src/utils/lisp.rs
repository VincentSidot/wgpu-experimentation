#![allow(unused_macros, unused_imports)]

macro_rules! prog1 {
    ($first:expr; $($rest:expr);*;) => {{
        let __prog1_result = $first;
        $(
            $rest;
        )*
        __prog1_result
    }};
}

macro_rules! prog2 {
    ($first:expr; $($rest:expr);*;) => {{
        $first;
        prog1!($($rest);*;)
    }};
}

macro_rules! setq {
    ($value: expr => $name: expr) => {{
        $name = $value;
        $name
    }};
}

// macro_rules! prog2 {
//     ($first:expr; $($rest:expr);*) => {{
//         $first;
//         prog1!($($rest);*)
//     }};
// }

pub(crate) use prog1;
pub(crate) use prog2;
pub(crate) use setq;

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_prog1() {
        let result = prog1!(1; 2; 3; 4; 5;);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_prog2() {
        let result = prog2!(1; 2; 3; 4; 5;);
        assert_eq!(result, 2);
    }
}
