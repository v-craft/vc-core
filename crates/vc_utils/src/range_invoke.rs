/// Call the target macro and pass a sequence of numbers as parameters.
///
/// The number cannot exceed `16` .
///
/// # Example
///
/// ```ignore
/// range_invoke!(my_macro,  4);
/// // eq  to ↓
/// my_macro!(0: []);
/// my_macro!(1: [P0]);
/// my_macro!(2: [P0, P1]);
/// my_macro!(3: [P0, P1, P2]);
/// my_macro!(4: [P0, P1, P2, P3]);
///
/// range_invoke!(my_macro,  4: P);
/// // eq  to ↓
/// my_macro!(0: []);
/// my_macro!(1: [0: P0]);
/// my_macro!(2: [0: P0, 1: P1]);
/// my_macro!(3: [0: P0, 1: P1, 2: P2]);
/// my_macro!(4: [0: P0, 1: P1, 2: P2, 3: P3]);
/// ```
#[macro_export]
macro_rules! range_invoke {
    ($(#[$meta:meta])* $macro:ident, 0: P) => {
        $(#[$meta])* $macro!(0: []);
    };
    ($(#[$meta:meta])* $macro:ident, 0) => {
        $(#[$meta])* $macro!(0: []);
    };
    ($(#[$meta:meta])* $macro:ident, 1: P) => {
        $(#[$meta])* $macro!(0: []);
        $(#[$meta])* $macro!(1: [0: P0]);
    };
    ($(#[$meta:meta])* $macro:ident, 1) => {
        $(#[$meta])* $macro!(0: []);
        $(#[$meta])* $macro!(1: [P0]);
    };
    ($(#[$meta:meta])* $macro:ident, 2: P) => {
        $(#[$meta])* $macro!(0: []);
        $(#[$meta])* $macro!(1: [0: P0]);
        $(#[$meta])* $macro!(2: [0: P0, 1: P1]);
    };
    ($(#[$meta:meta])* $macro:ident, 2) => {
        $(#[$meta])* $macro!(0: []);
        $(#[$meta])* $macro!(1: [P0]);
        $(#[$meta])* $macro!(2: [P0, P1]);
    };
    ($(#[$meta:meta])* $macro:ident, 3: P) => {
        $(#[$meta])* $macro!(0: []);
        $(#[$meta])* $macro!(1: [0: P0]);
        $(#[$meta])* $macro!(2: [0: P0, 1: P1]);
        $(#[$meta])* $macro!(3: [0: P0, 1: P1, 2: P2]);
    };
    ($(#[$meta:meta])* $macro:ident, 3) => {
        $(#[$meta])* $macro!(0: []);
        $(#[$meta])* $macro!(1: [P0]);
        $(#[$meta])* $macro!(2: [P0, P1]);
        $(#[$meta])* $macro!(3: [P0, P1, P2]);
    };
    ($(#[$meta:meta])* $macro:ident, 4: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3: P);
        $(#[$meta])* $macro!(4: [0: P0, 1: P1, 2: P2, 3: P3]);
    };
    ($(#[$meta:meta])* $macro:ident, 4) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3);
        $(#[$meta])* $macro!(4: [P0, P1, P2, P3]);
    };
    ($(#[$meta:meta])* $macro:ident, 5: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3: P);
        $(#[$meta])* $macro!(4: [0: P0, 1: P1, 2: P2, 3: P3]);
        $(#[$meta])* $macro!(5: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4]);
    };
    ($(#[$meta:meta])* $macro:ident, 5) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3);
        $(#[$meta])* $macro!(4: [P0, P1, P2, P3]);
        $(#[$meta])* $macro!(5: [P0, P1, P2, P3, P4]);
    };
    ($(#[$meta:meta])* $macro:ident, 6: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3: P);
        $(#[$meta])* $macro!(4: [0: P0, 1: P1, 2: P2, 3: P3]);
        $(#[$meta])* $macro!(5: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4]);
        $(#[$meta])* $macro!(6: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5]);
    };
    ($(#[$meta:meta])* $macro:ident, 6) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3);
        $(#[$meta])* $macro!(4: [P0, P1, P2, P3]);
        $(#[$meta])* $macro!(5: [P0, P1, P2, P3, P4]);
        $(#[$meta])* $macro!(6: [P0, P1, P2, P3, P4, P5]);
    };
    ($(#[$meta:meta])* $macro:ident, 7: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3: P);
        $(#[$meta])* $macro!(4: [0: P0, 1: P1, 2: P2, 3: P3]);
        $(#[$meta])* $macro!(5: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4]);
        $(#[$meta])* $macro!(6: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5]);
        $(#[$meta])* $macro!(7: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6]);
    };
    ($(#[$meta:meta])* $macro:ident, 7) => {
        $crate::range_invoke!($(#[$meta])* $macro, 3);
        $(#[$meta])* $macro!(4: [P0, P1, P2, P3]);
        $(#[$meta])* $macro!(5: [P0, P1, P2, P3, P4]);
        $(#[$meta])* $macro!(6: [P0, P1, P2, P3, P4, P5]);
        $(#[$meta])* $macro!(7: [P0, P1, P2, P3, P4, P5, P6]);
    };
    ($(#[$meta:meta])* $macro:ident, 8: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7: P);
        $(#[$meta])* $macro!(8: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7]);
    };
    ($(#[$meta:meta])* $macro:ident, 8) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7);
        $(#[$meta])* $macro!(8: [P0, P1, P2, P3, P4, P5, P6, P7]);
    };
    ($(#[$meta:meta])* $macro:ident, 9: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7: P);
        $(#[$meta])* $macro!(8: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7]);
        $(#[$meta])* $macro!(9: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8]);
    };
    ($(#[$meta:meta])* $macro:ident, 9) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7);
        $(#[$meta])* $macro!(8: [P0, P1, P2, P3, P4, P5, P6, P7]);
        $(#[$meta])* $macro!(9: [P0, P1, P2, P3, P4, P5, P6, P7, P8]);
    };
    ($(#[$meta:meta])* $macro:ident, 10: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7: P);
        $(#[$meta])* $macro!(8: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7]);
        $(#[$meta])* $macro!(9: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8]);
        $(#[$meta])* $macro!(10: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9]);
    };
    ($(#[$meta:meta])* $macro:ident, 10) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7);
        $(#[$meta])* $macro!(8: [P0, P1, P2, P3, P4, P5, P6, P7]);
        $(#[$meta])* $macro!(9: [P0, P1, P2, P3, P4, P5, P6, P7, P8]);
        $(#[$meta])* $macro!(10: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9]);
    };
    ($(#[$meta:meta])* $macro:ident, 11: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7: P);
        $(#[$meta])* $macro!(8: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7]);
        $(#[$meta])* $macro!(9: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8]);
        $(#[$meta])* $macro!(10: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9]);
        $(#[$meta])* $macro!(11: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10]);
    };
    ($(#[$meta:meta])* $macro:ident, 11) => {
        $crate::range_invoke!($(#[$meta])* $macro, 7);
        $(#[$meta])* $macro!(8: [P0, P1, P2, P3, P4, P5, P6, P7]);
        $(#[$meta])* $macro!(9: [P0, P1, P2, P3, P4, P5, P6, P7, P8]);
        $(#[$meta])* $macro!(10: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9]);
        $(#[$meta])* $macro!(11: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10]);
    };
    ($(#[$meta:meta])* $macro:ident, 12: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11: P);
        $(#[$meta])* $macro!(12: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11]);
    };
    ($(#[$meta:meta])* $macro:ident, 12) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11);
        $(#[$meta])* $macro!(12: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11]);
    };
    ($(#[$meta:meta])* $macro:ident, 13: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11: P);
        $(#[$meta])* $macro!(12: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11]);
        $(#[$meta])* $macro!(13: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11, 12: P12]);
    };
    ($(#[$meta:meta])* $macro:ident, 13) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11);
        $(#[$meta])* $macro!(12: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11]);
        $(#[$meta])* $macro!(13: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12]);
    };
    ($(#[$meta:meta])* $macro:ident, 14: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11: P);
        $(#[$meta])* $macro!(12: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11]);
        $(#[$meta])* $macro!(13: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11, 12: P12]);
        $(#[$meta])* $macro!(14: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11, 12: P12, 13: P13]);
    };
    ($(#[$meta:meta])* $macro:ident, 14) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11);
        $(#[$meta])* $macro!(12: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11]);
        $(#[$meta])* $macro!(13: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12]);
        $(#[$meta])* $macro!(14: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13]);
    };
    ($(#[$meta:meta])* $macro:ident, 15: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11: P);
        $(#[$meta])* $macro!(12: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11]);
        $(#[$meta])* $macro!(13: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11, 12: P12]);
        $(#[$meta])* $macro!(14: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11, 12: P12, 13: P13]);
        $(#[$meta])* $macro!(15: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11, 12: P12, 13: P13, 14: P14]);
    };
    ($(#[$meta:meta])* $macro:ident, 15) => {
        $crate::range_invoke!($(#[$meta])* $macro, 11);
        $(#[$meta])* $macro!(12: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11]);
        $(#[$meta])* $macro!(13: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12]);
        $(#[$meta])* $macro!(14: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13]);
        $(#[$meta])* $macro!(15: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14]);
    };
    ($(#[$meta:meta])* $macro:ident, 16: P) => {
        $crate::range_invoke!($(#[$meta])* $macro, 15: P);
        $(#[$meta])* $macro!(16: [0: P0, 1: P1, 2: P2, 3: P3, 4: P4, 5: P5, 6: P6, 7: P7, 8: P8, 9: P9, 10: P10, 11: P11, 12: P12, 13: P13, 14: P14, 15: P15]);
    };
    ($(#[$meta:meta])* $macro:ident, 16) => {
        $crate::range_invoke!($(#[$meta])* $macro, 15);
        $(#[$meta])* $macro!(16: [P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15]);
    };
}

/*
use std::{fs, io::Write};

const MAX_NUM: i32 = 16;
const STEP_SIZE: i32 = 4;

fn main () -> std::io::Result<()> {
    let mut file =  fs::OpenOptions::new()
        .create(true).write(true).truncate(true)
        .open("result.txt")?;

    file.write(b"#[macro_export]\n")?;
    file.write(b"macro_rules! range_invoke {\n")?;


    for i in 0..=MAX_NUM {
        let k = (i / STEP_SIZE) * STEP_SIZE - 1;

        file.write_fmt(format_args!("    ($(#[$meta:meta])* $macro:ident, {i}: P) => {{\n"))?;
        if k > 0 {
            file.write_fmt(format_args!("        $crate::range_invoke!($(#[$meta])* $macro, {k}: P);\n"))?;
        }
        for it in k+1..=i {
            file.write_fmt(format_args!("        $(#[$meta])* $macro!({it}: ["))?;
            if it > 0 {
                file.write(b"0: P0")?;
                for cnt in 1..it {
                    file.write_fmt(format_args!(", {cnt}: P{cnt}"))?;
                }
            }
            file.write(b"]);\n")?;
        }
        file.write(b"    };\n")?;
        file.write_fmt(format_args!("    ($(#[$meta:meta])* $macro:ident, {i}) => {{\n"))?;
        if k > 0 {
            file.write_fmt(format_args!("        $crate::range_invoke!($(#[$meta])* $macro, {k});\n"))?;
        }
        for it in k+1..=i {
            file.write_fmt(format_args!("        $(#[$meta])* $macro!({it}: ["))?;
            if it > 0 {
                file.write(b"P0")?;
                for cnt in 1..it {
                    file.write_fmt(format_args!(", P{cnt}"))?;
                }
            }
            file.write(b"]);\n")?;
        }
        file.write(b"    };\n")?;
    }
    file.write(b"}\n")?;

    file.flush()?;

    Ok(())
}
*/
