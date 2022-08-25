use crate::common::util::*;

#[test]
fn test_chr() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}

#[test]
fn test_chr2() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}

#[test]
fn test_chr3() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}

#[test]
fn test_metadata() {
    for _ in 0..100 {
        let (at, mut ucmd) = at_and_ucmd!();
        at.touch("hey");
        ucmd.arg("hey").succeeds();
    }
}
