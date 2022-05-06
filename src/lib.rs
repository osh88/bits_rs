//! A library for working with bit sequences

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]

use std::ops::{BitAnd, Shr, Shl, BitOrAssign};
use num::{Integer};
use std::convert::TryFrom;

/// Принимает на вход битовую последовательность (src.len() * bits_in),
/// упакованную в срез целых чисел (src), по bits_in бит в каждом эл-те.
/// Из src.len()*bits_in использует только bits_limit бит.
/// Упаковывает данную битовую последовательность в срез целых чисел
/// по bits_out бит в каждом эл-те. Типы эл-тов входного и выходного
/// срезов могут быть разными, главное чтобы bits_in и bits_out были меньше или равны
/// размерам соответствующих эл-тов.
///
/// &[5u16, 5u16] -> &[2u8, 3u8, 1u8]
/// в исходном срезе размер элементов 16 бит, из них значащих 3 (bits_in).
/// в результирующем срезе размер элементов 8 бит, из них значащих будет по 2 бита (bits_out).
/// то есть в данных [0b00000000_00000101u16, 0b00000000_00000101u16] значащая
/// битовая последовательность 101101, ее раскидываем в массив u8 по два бита на элемент:
/// 10 11 01 -> [0b00000010u8, 0b00000011u8, 0b00000001u8].
///
/// Если кол-во всех значащих бит в исходном срезе меньше bits_limit, недостающие биты
/// заполняются нулями.
///
/// # Arguments
/// * `src` - срез с данными.
/// * `bits_in` - кол-во значащих бит (справа) в каждом эл-те входного среза.
/// * `bits_out` - кол-во значащих бит (справа) в каждом эл-те выходного среза.
/// * `bits_limit` - ограничение кол-ва всех входных значащих битов.
///
/// # Errors
/// * `Err("bits_in < 1 || bits_out < 1 || bits_limit < 1")`
/// * `Err("bits_in > T1::size")`
/// * `Err("bits_out > T2::size")`
/// * `Err("bits_limit % bits_out != 0")`
/// * `Err("can't convert usize to T1")`
/// * `Err("can't convert usize to T2")`
/// * `Err("can't convert T1 to T2")`
///
/// # Examples
///
/// ```
///     let src = [5u16, 5]; // [0b_101, 0b_101]
///     let dst = [2u8, 3, 1]; // [0b_10, 0b_11, 0b_01]
///     let r: Vec<u8> = bits::repack(&src, 3, 2, 6).unwrap();
///     assert_eq!(dst, r.as_slice());
/// ```
///
/// ```
///     let src = [5u16, 5]; // [0b_101, 0b_101]
///     let dst = [11u8, 4]; // [0b_1011, 0b_0100]
///     let r: Vec<u8> = bits::repack(&src, 3, 4, 8).unwrap();
///     assert_eq!(dst, r.as_slice());
/// ```
pub fn repack<T1, T2>(src: &[T1], bits_in: usize, bits_out: usize, bits_limit: usize) -> Result<Vec<T2>, &'static str>
where
    T1: BitAnd<Output = T1> + Integer + Clone + Shr<Output = T1> + TryFrom<usize>,
    T2: Integer + Clone + TryFrom<T1> + BitOrAssign + TryFrom<usize> + Shl<Output = T2>,
{
    if bits_in < 1 || bits_out < 1 || bits_limit < 1 {
        return Err("bits_in < 1 || bits_out < 1 || bits_limit < 1");
    }

    let src_bit_size = std::mem::size_of_val(&T1::zero()) * 8;
    if bits_in > src_bit_size {
        return Err("bits_in > T1::size");
    }

    let dst_bit_size = std::mem::size_of_val(&T2::zero()) * 8;
    if bits_out > dst_bit_size {
        return Err("bits_out > T2::size");
    }

    if bits_limit % bits_out != 0 {
        return Err("bits_limit % bits_out != 0")
    }

    let mut dst = vec![T2::zero(); bits_limit / bits_out];
    for i in 0..bits_limit {
        // Номер входного байта
        let src_i = i / bits_in;
        // Номер входного бита в байте
        let src_b = i % bits_in;
        // Номер выходного байта
        let dst_i = i / bits_out;
        // Номер выходного бита в байте
        let dst_b = i % bits_out;

        // Сдвиг нужного бита в нулевую позицию.
        let rsh = match T1::try_from(bits_in - src_b - 1) {
            Ok(v) => v,
            Err(_) => return Err("can't convert usize to T1"),
        };

        // Сдвиг бита влево в нужную позицию.
        let lsh = match T2::try_from(bits_out - dst_b - 1) {
            Ok(v) => v,
            Err(_) => return Err("can't convert usize to T2"),
        };

        // Если кол-во бит в результате превышает кол-во входных бит, дополняем
        // результат нулевыми битами.
        let src_byte = if src_i < src.len() {
            src[src_i].clone()
        } else {
            T1::zero()
        };

        // Текущий бит в текущем байте сдвигаем вправо и обрезаем до 1 бита.
        // Получили значение исходного бита, затем сдвигаем его на позицию,
        // в которой он должен находиться в выходном байте.
        let src_bit = match T2::try_from((src_byte >> rsh) & T1::one()) {
            Ok(v) => v,
            Err(_) => return Err("can't convert T1 to T2"),
        };

        // Копируем бит в выходной байт.
        dst[dst_i] |= src_bit << lsh;
    }

    Ok(dst)
}

#[test]
fn test1() {
    let src = [0b_00101001_00010000_u16, 0b_00101001_00010000_u16];
    let dst = [0b_00101001_u8, 0b_00010000_u8, 0b_00101001u8, 0b_00010000_u8];
    let r: Vec<u8> = repack(&src, 16, 8, 32).unwrap();
    assert_eq!(dst, r.as_slice());
}

#[test]
fn test2() {
    let src = [0xFF, 0xFF];
    let dst = [0u8, 0, 0, 0xFF, 0, 0, 0, 0xFF];
    let r: Vec<u8> = repack(&src, 32, 8, 64).unwrap();
    assert_eq!(dst, r.as_slice());
}

// Общее кол-во выходных бит нельзя поровну разделить на кол-во бит в выходном эл-те.
#[test]
#[should_panic(expected = "bits_limit % bits_out != 0")]
fn test3() {
    let src = [0xFF, 0xFF];
    let dst = [0u8, 0, 0, 0xFF, 0, 0, 0, 0xFF];
    let r: Vec<u8> = repack(&src, 32, 7, 64).unwrap();
    assert_eq!(dst, r.as_slice());
}

// Кол-во значащих входных бит в одном эл-те превышает размер входного элемента.
#[test]
#[should_panic(expected = "bits_in > T1::size")]
fn test4() {
    let src = [0xFF, 0xFF];
    let _: Vec<u8> = repack(&src, 256, 7, 64).unwrap();
}

// Кол-во значащих выходных бит в одном эл-те превышает размер выходного элемента.
#[test]
#[should_panic(expected = "bits_out > T2::size")]
fn test5() {
    let src = [0xFF, 0xFF];
    let _: Vec<u8> = repack(&src, 32, 16, 64).unwrap();
}

// Недопустимое значение bits_in.
#[test]
#[should_panic(expected = "bits_in < 1 || bits_out < 1 || bits_limit < 1")]
fn test6() {
    let src = [0xFF, 0xFF];
    let _: Vec<u8> = repack(&src, 0, 16, 64).unwrap();
}

// Недопустимое значение bits_out.
#[test]
#[should_panic(expected = "bits_in < 1 || bits_out < 1 || bits_limit < 1")]
fn test7() {
    let src = [0xFF, 0xFF];
    let _: Vec<u8> = repack(&src, 32, 0, 64).unwrap();
}

// Недопустимое значение bits_limit.
#[test]
#[should_panic(expected = "bits_in < 1 || bits_out < 1 || bits_limit < 1")]
fn test8() {
    let src = [0xFF, 0xFF];
    let _: Vec<u8> = repack(&src, 32, 16, 0).unwrap();
}

#[test]
fn test9() {
    let src = [0b_00101001_u8, 0b_00010000_u8, 0b_00101001u8, 0b_00010000_u8];
    let dst = [0b_00101001_00010000_u16, 0b_00101001_00010000_u16];
    let r: Vec<u16> = repack(&src, 8, 16, 32).unwrap();
    assert_eq!(dst, r.as_slice());
}

#[test]
fn test10() {
    let src = [5u16, 5]; // [0b_101, 0b_101]
    let dst = [11u8, 4]; // [0b_1011, 0b_0100]
    let r: Vec<u8> = repack(&src, 3, 4, 8).unwrap();
    assert_eq!(dst, r.as_slice());
}
