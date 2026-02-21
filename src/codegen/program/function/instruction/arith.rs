// TODO arithmetic operations
// for bitness in [32, 64] {
//     b.def(
//         format!("ToBitsBE{bitness}"),
//         abs(
//             (0..bitness).rev().map(|i| i.to_string()),
//             unsafe_list::from((0..bitness).rev().map(|i| var(i.to_string()))),
//         ),
//     );
// }

// pub fn to_bit_list_be(bitness: u8, number: Number) -> unsafe_list::UnsafeList {
//     debug_assert!(bitness == 16 || bitness == 32);

//     apply(number, [var(format!("ToBitsBE{bitness}"))])
// }
