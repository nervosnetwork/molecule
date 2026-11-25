use molecule::lazy_reader::{AccessError, Cursor, Error, Read, MAX_CACHE_SIZE};
use std::{cell::Cell, rc::Rc};

struct ShortSource {
    bytes: Vec<u8>,
    chunk: usize,
    calls: Rc<Cell<usize>>,
}

impl Read for ShortSource {
    fn read(&self, output: &mut [u8], offset: usize) -> Result<usize, Error> {
        self.calls.set(self.calls.get() + 1);
        let remaining = self
            .bytes
            .get(offset..)
            .ok_or(Error::OutOfBound(offset, self.bytes.len()))?;
        let count = remaining.len().min(output.len()).min(self.chunk);
        output[..count].copy_from_slice(&remaining[..count]);
        Ok(count)
    }
}

fn cursor(bytes: Vec<u8>, chunk: usize) -> (Cursor, Rc<Cell<usize>>) {
    let calls = Rc::new(Cell::new(0));
    let size = bytes.len();
    (
        Cursor::new(
            size,
            Box::new(ShortSource {
                bytes,
                chunk,
                calls: calls.clone(),
            }),
        ),
        calls,
    )
}

#[test]
fn exact_reads_retry_short_sources_in_both_paths() {
    for size in [8, MAX_CACHE_SIZE + 31] {
        let bytes: Vec<_> = (0..size).map(|i| i as u8).collect();
        let (input, calls) = cursor(bytes.clone(), 3);
        let mut output = vec![0; size];
        input.read_exact_at(0, &mut output).unwrap();
        assert_eq!(output, bytes);
        assert_eq!(calls.get(), size.div_ceil(3));
    }
}

#[test]
fn logical_bounds_fail_before_io_or_destination_changes() {
    let (input, calls) = cursor(vec![7; MAX_CACHE_SIZE * 2], usize::MAX);
    let input = input.slice_exact(100, MAX_CACHE_SIZE + 7).unwrap();
    let mut output = vec![0xa5; input.size + 1];
    assert!(matches!(
        input.read_exact_at(0, &mut output),
        Err(AccessError::Bounds { .. })
    ));
    assert!(output.iter().all(|&byte| byte == 0xa5));
    assert_eq!(calls.get(), 0);
    assert!(matches!(
        input.read_array::<8>(),
        Err(AccessError::Read(Error::TotalSize(8, _)))
    ));
    assert_eq!(calls.get(), 0);
    assert!(matches!(
        input.slice_exact(usize::MAX, 1),
        Err(AccessError::Overflow)
    ));
}

#[test]
fn public_cursor_fields_do_not_bypass_absolute_overflow_checks() {
    let (mut input, calls) = cursor(vec![1; 8], 8);
    input.offset = usize::MAX - 3;
    assert!(matches!(
        input.read_array::<8>(),
        Err(AccessError::Overflow)
    ));
    assert_eq!(calls.get(), 0);
}

#[test]
fn empty_range_at_end_needs_no_source_access() {
    let (input, calls) = cursor(vec![1; 8], 8);
    input.read_exact_at(8, &mut []).unwrap();
    assert_eq!(calls.get(), 0);
    assert!(matches!(
        input.read_exact_at(9, &mut []),
        Err(AccessError::Bounds { .. })
    ));
}

#[test]
fn exact_reads_reuse_the_cache_and_report_borrow_conflicts() {
    let (input, calls) = cursor((0..8).collect(), 8);
    let first = input.read_array::<8>().unwrap();
    assert_eq!(input.read_array::<8>().unwrap(), first);
    assert_eq!(calls.get(), 1);

    let _borrow = input.data_source.borrow_mut();
    assert!(matches!(
        input.read_array::<8>(),
        Err(AccessError::BorrowConflict)
    ));
}

struct InvalidSource(bool);
impl Read for InvalidSource {
    fn read(&self, output: &mut [u8], _: usize) -> Result<usize, Error> {
        Ok(if self.0 { output.len() + 1 } else { 0 })
    }
}

#[test]
fn no_progress_and_impossible_read_counts_are_errors() {
    for size in [8, MAX_CACHE_SIZE + 1] {
        let mut output = vec![0; size];
        let zero = Cursor::new(size, Box::new(InvalidSource(false)));
        assert!(matches!(
            zero.read_exact_at(0, &mut output),
            Err(AccessError::NoProgress { .. })
        ));
        let excess = Cursor::new(size, Box::new(InvalidSource(true)));
        assert!(matches!(
            excess.read_exact_at(0, &mut output),
            Err(AccessError::InvalidRead { .. })
        ));
    }
}

struct FailsAfterWriting;
impl Read for FailsAfterWriting {
    fn read(&self, output: &mut [u8], offset: usize) -> Result<usize, Error> {
        for (index, byte) in output.iter_mut().enumerate() {
            *byte = (offset + index) as u8;
        }
        if offset == 32 {
            Err(Error::Read(0, output.len()))
        } else {
            Ok(output.len())
        }
    }
}

#[test]
fn failed_legacy_fill_cannot_poison_a_subsequent_bounded_read() {
    let input = Cursor::new(64, Box::new(FailsAfterWriting));
    let expected = input.slice_exact(0, 8).unwrap().read_array::<8>().unwrap();
    assert_eq!(expected, [0, 1, 2, 3, 4, 5, 6, 7]);
    assert!(input
        .slice_exact(32, 8)
        .unwrap()
        .read_at(&mut [0; 8])
        .is_err());
    assert_eq!(
        input.slice_exact(0, 8).unwrap().read_array::<8>().unwrap(),
        expected
    );
}

#[test]
fn checked_field_rejects_invalid_offsets() {
    for offsets in [[16u32, 12], [12, 21], [12, 11]] {
        let bytes = [
            20u32.to_le_bytes(),
            offsets[0].to_le_bytes(),
            offsets[1].to_le_bytes(),
            [0; 4],
            [0; 4],
        ]
        .concat();
        let (input, _) = cursor(bytes, 2);
        assert!(input.table_field_exact(2, 0).is_err());
    }
}

#[test]
fn checked_field_reads_only_header_and_selected_field_never_trailing_payloads() {
    let mut wire = [26u32, 16, 24, 25].map(u32::to_le_bytes).concat();
    wire.extend_from_slice(&61u64.to_le_bytes());
    wire.extend_from_slice(&[1, 2]);
    let (input, calls) = cursor(wire, usize::MAX);
    let amount = input.table_field_exact(3, 0).unwrap();
    assert_eq!(amount.read_array::<8>().unwrap(), 61u64.to_le_bytes());
    // Reads: total_size (0..4), first offset (4..8), remaining offsets (8..16), field (16..24).
    assert_eq!(calls.get(), 4);
    assert_eq!(
        input
            .table_field_exact(0, 2)
            .unwrap()
            .read_array::<1>()
            .unwrap(),
        [2]
    );
    // Header 0..16 remained cached via contiguous tail-append, so reading field 2 only fetches byte 25.
    assert_eq!(calls.get(), 5);
    assert!(input.table_field_exact(4, 0).is_err());
    assert!(input.table_field_exact(3, 3).is_err());

    // Empty 4-byte table reports FieldCount(0) when fields are required,
    // and OutOfBound when minimum_fields is 0.
    let (empty, _) = cursor(4u32.to_le_bytes().to_vec(), usize::MAX);
    assert!(matches!(
        empty.table_field_exact(1, 0),
        Err(AccessError::Read(Error::FieldCount(0)))
    ));
    assert!(matches!(
        empty.table_field_exact(0, 0),
        Err(AccessError::Read(Error::OutOfBound(0, 0)))
    ));

    // Reading an early field (e.g. CellOutput.capacity in bytes 0..24 of a 97-byte table)
    // must never request bytes past the header + selected field, even if trailing fields fail.
    struct PrefixOnlySource {
        prefix: Vec<u8>,
        max_end: Rc<Cell<usize>>,
    }
    impl Read for PrefixOnlySource {
        fn read(&self, output: &mut [u8], offset: usize) -> Result<usize, Error> {
            let end = offset.checked_add(output.len()).ok_or(Error::Overflow)?;
            self.max_end.set(self.max_end.get().max(end));
            if end > self.prefix.len() {
                return Err(Error::Read(self.prefix.len(), end));
            }
            output.copy_from_slice(&self.prefix[offset..end]);
            Ok(output.len())
        }
    }
    let mut cell_output_prefix = [97u32, 16, 24, 77].map(u32::to_le_bytes).concat();
    cell_output_prefix.extend_from_slice(&6_100_000_000u64.to_le_bytes());
    assert_eq!(cell_output_prefix.len(), 24);
    let max_end = Rc::new(Cell::new(0));
    let cell_output = Cursor::new(
        97,
        Box::new(PrefixOnlySource {
            prefix: cell_output_prefix,
            max_end: max_end.clone(),
        }),
    );
    let capacity = cell_output.table_field_exact(3, 0).unwrap();
    assert_eq!(
        capacity.read_array::<8>().unwrap(),
        6_100_000_000u64.to_le_bytes()
    );
    assert_eq!(max_end.get(), 24);

    // Tables with > 16 fields validate offsets in bounded stack chunks without touching payloads.
    let field_count = 20usize;
    let first = (field_count + 1) * 4;
    let total = first + field_count * 8;
    let mut wide_prefix = Vec::with_capacity(first + 8);
    wide_prefix.extend_from_slice(&(total as u32).to_le_bytes());
    for i in 0..field_count {
        wide_prefix.extend_from_slice(&((first + i * 8) as u32).to_le_bytes());
    }
    wide_prefix.extend_from_slice(&42u64.to_le_bytes());
    let wide_max = Rc::new(Cell::new(0));
    let wide = Cursor::new(
        total,
        Box::new(PrefixOnlySource {
            prefix: wide_prefix,
            max_end: wide_max.clone(),
        }),
    );
    let first_field = wide.table_field_exact(field_count, 0).unwrap();
    assert_eq!(first_field.read_array::<8>().unwrap(), 42u64.to_le_bytes());
    assert_eq!(wide_max.get(), first + 8);
}

#[test]
fn fixvec_item_exact_validates_header_bounds_and_skips_other_items() {
    let mut wire = 3u32.to_le_bytes().to_vec();
    for v in [10u64, 20, 30] {
        wire.extend_from_slice(&v.to_le_bytes());
    }
    let (input, calls) = cursor(wire.clone(), usize::MAX);
    assert_eq!(
        input
            .fixvec_item_exact(8, 0)
            .unwrap()
            .read_array::<8>()
            .unwrap(),
        10u64.to_le_bytes()
    );
    assert_eq!(calls.get(), 2);
    assert_eq!(
        input
            .fixvec_item_exact(8, 2)
            .unwrap()
            .read_array::<8>()
            .unwrap(),
        30u64.to_le_bytes()
    );
    // Count header [0..4) remained cached after contiguous item 0 read, so item 2 adds only 1 read.
    assert_eq!(calls.get(), 3);
    assert!(matches!(
        input.fixvec_item_exact(8, 3),
        Err(AccessError::Read(Error::OutOfBound(3, 3)))
    ));
    assert!(matches!(
        input.fixvec_item_exact(0, 0),
        Err(AccessError::Read(Error::Data))
    ));
    // Truncated length fails before accessing an item.
    let (short, _) = cursor(wire[..wire.len() - 1].to_vec(), 2);
    assert!(matches!(
        short.fixvec_item_exact(8, 0),
        Err(AccessError::Read(Error::TotalSize(_, _)))
    ));
}

#[test]
fn legacy_read_at_rejects_overflow_and_impossible_read_counts() {
    let mut overflow = Cursor::from([1u8; 8]);
    overflow.offset = usize::MAX;
    assert!(matches!(
        overflow.read_at(&mut [0u8; 1]),
        Err(Error::Overflow)
    ));

    let bad = Cursor::new(MAX_CACHE_SIZE + 8, Box::new(InvalidSource(true)));
    assert!(matches!(
        bad.read_at(&mut [0u8; 4]),
        Err(Error::Read(_, MAX_CACHE_SIZE))
    ));
    assert!(matches!(
        bad.read_at(&mut vec![0u8; MAX_CACHE_SIZE + 1]),
        Err(Error::Read(_, _))
    ));
}

#[test]
fn access_error_converts_into_lazy_error_and_exact_helpers_retry_short_reads() {
    assert!(matches!(
        Error::from(AccessError::Read(Error::TotalSize(8, 3))),
        Error::TotalSize(8, 3)
    ));
    assert!(matches!(
        Error::from(AccessError::Bounds {
            offset: 4,
            len: 8,
            size: 10
        }),
        Error::OutOfBound(12, 10)
    ));
    assert!(matches!(
        Error::from(AccessError::Overflow),
        Error::Overflow
    ));
    assert!(matches!(
        Error::from(AccessError::NoProgress {
            completed: 2,
            required: 8
        }),
        Error::Read(2, 8)
    ));
    assert!(matches!(
        Error::from(AccessError::InvalidRead {
            reported: 9,
            capacity: 8
        }),
        Error::Read(9, 8)
    ));
    assert!(matches!(
        Error::from(AccessError::BorrowConflict),
        Error::Common
    ));

    let pair_wire = [10u64, 20].map(u64::to_be_bytes).concat();
    let (pair, _) = cursor(pair_wire, 3);
    assert_eq!(
        pair.array_item_exact(2, 8, 1)
            .unwrap()
            .read_array::<8>()
            .unwrap(),
        20u64.to_be_bytes()
    );
    assert!(matches!(
        pair.array_item_exact(2, 8, 2),
        Err(AccessError::Read(Error::OutOfBound(2, 2)))
    ));
    assert!(matches!(
        pair.array_item_exact(3, 8, 0),
        Err(AccessError::Read(Error::TotalSize(24, 16)))
    ));
    assert!(matches!(
        pair.array_item_exact(usize::MAX, 2, 0),
        Err(AccessError::Overflow)
    ));
    assert!(matches!(
        pair.array_item_exact(2, 0, 0),
        Err(AccessError::Read(Error::Data))
    ));
    assert_eq!(
        pair.fixed_field_exact(16, 8, 8)
            .unwrap()
            .read_array::<8>()
            .unwrap(),
        20u64.to_be_bytes()
    );
    assert!(matches!(
        pair.fixed_field_exact(15, 0, 8),
        Err(AccessError::Read(Error::TotalSize(15, 16)))
    ));
    assert!(matches!(
        pair.fixed_field_exact(16, 12, 8),
        Err(AccessError::Bounds {
            offset: 12,
            len: 8,
            size: 16
        })
    ));

    let vec_wire = [
        2u32.to_le_bytes().as_slice(),
        &10u64.to_be_bytes(),
        &20u64.to_be_bytes(),
    ]
    .concat();
    let (vec_cur, _) = cursor(vec_wire, 3);
    assert_eq!(vec_cur.fixvec_len_exact(8).unwrap(), 2);

    let (empty_dyn, _) = cursor(4u32.to_le_bytes().to_vec(), 3);
    assert_eq!(empty_dyn.dynvec_len_exact().unwrap(), 0);
    assert_eq!(empty_dyn.verify_dynvec_exact().unwrap(), 0);
    assert_eq!(empty_dyn.verify_table_exact(0, false).unwrap(), 0);
    assert!(matches!(
        empty_dyn.verify_table_exact(1, true),
        Err(AccessError::Read(Error::FieldCount(0)))
    ));
    let dyn_wire = [16u32, 12, 12, 16].map(u32::to_le_bytes).concat();
    let (dyn_cur, _) = cursor(dyn_wire, 3);
    assert_eq!(dyn_cur.dynvec_len_exact().unwrap(), 2);
    assert_eq!(dyn_cur.verify_dynvec_exact().unwrap(), 2);
    assert_eq!(dyn_cur.verify_table_exact(2, false).unwrap(), 2);
    assert_eq!(dyn_cur.verify_table_exact(1, true).unwrap(), 2);
    assert!(matches!(
        dyn_cur.verify_table_exact(1, false),
        Err(AccessError::Read(Error::FieldCount(2)))
    ));
    assert_eq!(dyn_cur.dynvec_item_exact(2, 0).unwrap().size, 0);
    assert_eq!(dyn_cur.dynvec_item_exact(2, 1).unwrap().size, 4);
    assert_eq!(dyn_cur.dynvec_get_exact(0).unwrap().size, 0);
    assert_eq!(dyn_cur.dynvec_get_exact(1).unwrap().size, 4);
    assert!(matches!(
        dyn_cur.dynvec_get_exact(2),
        Err(AccessError::Read(Error::OutOfBound(2, 2)))
    ));
    assert!(matches!(
        empty_dyn.dynvec_get_exact(0),
        Err(AccessError::Read(Error::OutOfBound(0, 0)))
    ));
    assert!(matches!(
        dyn_cur.dynvec_item_exact(2, 2),
        Err(AccessError::Read(Error::OutOfBound(2, 2)))
    ));
    let bad_start = [12u32, 8, 12].map(u32::to_le_bytes).concat();
    let (bad_cur, _) = cursor(bad_start, 3);
    assert!(matches!(
        bad_cur.dynvec_item_exact(2, 0),
        Err(AccessError::Read(Error::Offset))
    ));

    let union_wire = [7u32.to_le_bytes().as_slice(), &[0xaa, 0xbb]].concat();
    let (union_cur, _) = cursor(union_wire, 3);
    let unpacked = union_cur.union_unpack_exact().unwrap();
    assert_eq!(unpacked.item_id, 7);
    assert_eq!(unpacked.cursor.read_array::<2>().unwrap(), [0xaa, 0xbb]);
    let (short_union, _) = cursor(vec![1, 2, 3], 2);
    assert!(matches!(
        short_union.union_unpack_exact(),
        Err(AccessError::Read(Error::Header(4, 3)))
    ));

    let raw_bytes_wire = [3u32.to_le_bytes().as_slice(), &[9, 8, 7]].concat();
    let (raw_cur, _) = cursor(raw_bytes_wire, 3);
    assert_eq!(
        raw_cur
            .fixvec_raw_bytes_exact()
            .unwrap()
            .read_array::<3>()
            .unwrap(),
        [9, 8, 7]
    );
    let (bad_raw, _) = cursor([0u32.to_le_bytes().as_slice(), &[9]].concat(), 2);
    assert!(matches!(
        bad_raw.fixvec_raw_bytes_exact(),
        Err(AccessError::Read(Error::TotalSize(4, 5)))
    ));
}
