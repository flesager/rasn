//! Aligned PER (X.691) length determinants and alignment, checked against
//! encodings from 3GPP NGAP (TS 38.413) as produced by other implementations.

use rasn::prelude::*;

#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
struct Item {
    a: FixedOctetString<3>,
    b: Option<u8>,
}

/// SIZE(1..12): 4-bit length, sharing its octet with the first item's preamble.
#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(delegate, size("1..=12"))]
struct SmallList(SequenceOf<Item>);

/// SIZE(0..65535): two-octet length, octet-aligned.
#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
struct WithLargeList {
    #[rasn(size("0..=65535"))]
    items: SequenceOf<u8>,
}

/// SIZE(1..256): one-octet length, octet-aligned.
#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
struct WithList256 {
    #[rasn(size("1..=256"))]
    items: SequenceOf<u8>,
}

/// SIZE(1..65536): "ub" is not less than 64K, a general length determinant.
#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
struct With64KList {
    #[rasn(size("1..=65536"))]
    items: SequenceOf<u8>,
}

#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
struct WithKey {
    #[rasn(value("0..=7"))]
    count: u8,
    key: FixedBitString<256>,
    ext: Option<u8>,
}

fn round_trip<T: Encode + Decode + PartialEq + core::fmt::Debug>(value: &T, expected: &[u8]) {
    let encoded = rasn::aper::encode(value).unwrap();
    assert_eq!(encoded, expected, "encoding");
    assert_eq!(
        &rasn::aper::decode::<T>(expected).unwrap(),
        value,
        "decoding"
    );
}

#[test]
fn sequence_of_components_follow_a_bit_field_length_without_alignment() {
    // length 1 (4 bits: 0000), item preamble (2 bits: 00), padding, then `a`.
    let list = SmallList(vec![Item {
        a: FixedOctetString::new([0x02, 0xf8, 0x39]),
        b: None,
    }]);
    round_trip(&list, &[0x00, 0x02, 0xf8, 0x39]);
}

#[test]
fn two_octet_lengths_are_octet_aligned() {
    // extension bit, padding, then 0x0004 (NGAP: an IE container of 4 IEs).
    let value = WithLargeList {
        items: vec![1, 2, 3, 4],
    };
    round_trip(&value, &[0x00, 0x00, 0x04, 1, 2, 3, 4]);
}

#[test]
fn one_octet_lengths_of_range_256_are_octet_aligned() {
    // extension bit, padding, then length - 1 = 0x01.
    let value = WithList256 { items: vec![7, 8] };
    round_trip(&value, &[0x00, 0x01, 7, 8]);
}

#[test]
fn an_upper_bound_of_64k_uses_a_general_length() {
    // extension bit, padding, then the general form: one octet, 1.
    // (NGAP UE-associatedLogicalNG-connectionList, SIZE(1..65536).)
    let value = With64KList { items: vec![9] };
    round_trip(&value, &[0x00, 0x01, 9]);
}

#[test]
fn fixed_bit_strings_above_16_bits_are_octet_aligned_when_decoding() {
    // NGAP SecurityContext: extension bit, optional bit, 3-bit count = 2,
    // padding, then the 256-bit key from the next octet.
    let mut key = FixedBitString::<256>::default();
    key.set(0, true);
    let value = WithKey {
        count: 2,
        key,
        ext: None,
    };
    let mut expected = vec![0x10, 0x80];
    expected.resize(33, 0);
    round_trip(&value, &expected);
}
