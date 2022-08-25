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
fn test_chr11() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}


#[test]
fn test_chr4() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}


#[test]
fn test_chr5() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}


#[test]
fn test_chr6() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}


#[test]
fn test_chr7() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}


#[test]
fn test_chr8() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}


#[test]
fn test_chr9() {
    for _ in 0..100 {
        new_ucmd!().succeeds();
    }
}


#[test]
fn test_chr10() {
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
