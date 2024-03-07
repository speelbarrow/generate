#![allow(dead_code)]

struct Solution;
impl Solution {
    pub fn {{ crate_name }} {{ signature }} {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use paste::paste;
    use super::*;

    macro_rules! example {
        ($number: literal, $actual: expr, $( $arg: expr ),*) => {
            paste! {
                #[test]
                fn [<example $number>]() {
                    assert_eq!(Solution::{{ crate_name }}($($arg),*), $actual)
                }
            }
        };
    }

    {{ examples }}
}
