use crate::{lazy, packed};
use molecule::{lazy_reader::Cursor, prelude::*};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
    vec,
};

struct Counter;
std::thread_local! { static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) }; }
fn record() {
    let _ = ALLOCATIONS.try_with(|count| {
        if let Some(value) = count.get() {
            count.set(Some(value + 1));
        }
    });
}
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record();
        System.alloc(layout)
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record();
        System.alloc_zeroed(layout)
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record();
        System.realloc(pointer, layout, new_size)
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout)
    }
}
#[global_allocator]
static ALLOCATOR: Counter = Counter;

#[test]
fn numeric_vector_construction_allocates_only_its_wire_buffer() {
    use crate::{blockchain::Uint64, extensions::Uint64Vec};
    let values = [u64::MAX; 1024];
    for count in [0, 1, 32, 1024] {
        let values = &values[..count];
        let expected = Uint64Vec::new_builder()
            .extend(values.iter().copied().map(Uint64::from_u64))
            .build();
        ALLOCATIONS.with(|n| n.set(Some(0)));
        let packed = Uint64Vec::try_from_values(values).unwrap();
        let allocations = ALLOCATIONS.with(|n| n.replace(None).unwrap());
        assert_eq!(packed.as_slice(), expected.as_slice());
        assert_eq!(allocations, 1);
    }

    use crate::protocols::PortOpt;
    ALLOCATIONS.with(|n| n.set(Some(0)));
    let none = PortOpt::from_value(None);
    let none_allocs = ALLOCATIONS.with(|n| n.replace(None).unwrap());
    assert!(none.as_slice().is_empty());
    assert_eq!(none_allocs, 0);

    ALLOCATIONS.with(|n| n.set(Some(0)));
    let some = PortOpt::from_value(Some(8115));
    let some_allocs = ALLOCATIONS.with(|n| n.replace(None).unwrap());
    assert_eq!(some.as_slice(), 8115u16.to_le_bytes());
    assert_eq!(some_allocs, 1);
}

#[test]
fn scalar_and_container_reads_allocate_nothing() {
    let bytes = 123u64.to_be_bytes();
    let reader = packed::U64BeReader::new_unchecked(&bytes);
    let lazy = lazy::U64Be::from(Cursor::from(vec![0, 0, 0, 0, 0, 0, 0, 123]));
    let wide_bytes = u128::MAX.to_be_bytes();
    let wide_reader = packed::U128BeReader::new_unchecked(&wide_bytes);
    let port = crate::protocols::PortOptReader::new_unchecked(&[0, 1]);
    let fees =
        crate::extensions::Uint64VecReader::new_unchecked(&[1, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0]);
    ALLOCATIONS.with(|count| count.set(Some(0)));
    for _ in 0..1000 {
        assert_eq!(black_box(reader).try_to_u64().unwrap(), 123);
        assert_eq!(black_box(&lazy).try_to_u64().unwrap(), 123);
        assert_eq!(black_box(wide_reader).try_to_u128().unwrap(), u128::MAX);
        assert_eq!(black_box(port).try_value().unwrap(), Some(256));
        assert_eq!(black_box(fees).try_values().unwrap().next(), Some(123));
    }
    let allocations = ALLOCATIONS.with(|count| count.replace(None).unwrap());
    assert_eq!(allocations, 0);
}
