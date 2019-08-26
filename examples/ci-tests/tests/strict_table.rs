use molecule::prelude::*;

use molecule_ci_tests::types;

#[test]
fn strict_table() {
    let a = types::StrictTableA::default();
    let b = types::StrictTableB::default();
    let c = types::StrictTableC::default();

    assert!(types::StrictTableAReader::verify(a.as_slice()).is_ok());
    assert!(types::StrictTableAReader::verify(b.as_slice()).is_err());
    assert!(types::StrictTableAReader::verify(c.as_slice()).is_err());

    assert!(types::StrictTableBReader::verify(a.as_slice()).is_err());
    assert!(types::StrictTableBReader::verify(b.as_slice()).is_ok());
    assert!(types::StrictTableBReader::verify(c.as_slice()).is_err());

    assert!(types::StrictTableCReader::verify(a.as_slice()).is_err());
    assert!(types::StrictTableCReader::verify(b.as_slice()).is_err());
    assert!(types::StrictTableCReader::verify(c.as_slice()).is_ok());
}
