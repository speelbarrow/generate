pub fn {{ crate_name }} {{ signature }} {
    todo!()
}

#[cfg(test)]
mod tests {

    macro_rules! example {
        ($number: literal, $actual: expr, $( $arg: expr ),*) => {
            ::paste::paste! {
                #[test]
                fn [<example $number>]() {
                    assert_eq!(super::{{ crate_name }}($($arg),*), $actual)
                }
            }
        };
    }

    {{ examples }}
}
