#[cfg(test)]
mod tests {
    use molecule::prelude::*;

    static UNION_FOO_0_7_3_JSON_INTERMEDIATE: &str =
        include_str!(concat!(env!("OUT_DIR"), "/0_7_3/union_foo_0_7_3.json"));

    static UNION_FOO_DEV_JSON_INTERMEDIATE: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/dev/union_foo_with_custom_id.json"
    ));

    #[test]
    fn test_recover_0_7_3_intermediate_by_current_ir_recover() {
        let format = codegen_dev::IntermediateFormat::JSON;
        let ast_result = format.recover(UNION_FOO_0_7_3_JSON_INTERMEDIATE.as_bytes());
        assert!(ast_result.is_ok());
    }

    #[test]
    fn test_recover_ir() {
        let format = codegen_dev::IntermediateFormat::JSON;
        let ast_result = format.recover(UNION_FOO_DEV_JSON_INTERMEDIATE.as_bytes());
        assert!(ast_result.is_ok());
    }

    mod union_foo_0_7_3 {
        #![allow(clippy::all, dead_code)]
        include!(concat!(env!("OUT_DIR"), "/0_7_3/union_foo_0_7_3.rs"));
    }

    mod union_foo_dev {
        #![allow(clippy::all, dead_code)]
        include!(concat!(env!("OUT_DIR"), "/dev/union_foo_with_custom_id.rs"));
    }

    #[test]
    fn test_decode_0_7_3_generated_rust_bytes_by_current_version() {
        let a2_0_7_3 = union_foo_0_7_3::A2::new_builder()
            .nth0(Byte::from(17))
            .build();

        let foo_0_7_3 = union_foo_0_7_3::Foo::new_builder()
            .set(a2_0_7_3.clone())
            .build();
        let foo_0_7_3_slice = foo_0_7_3.as_slice();

        let foo_dev_result = union_foo_dev::FooOnlyReserveA2AndA3::from_slice(foo_0_7_3_slice);
        assert!(foo_dev_result.is_ok());
        let foo_dev = foo_dev_result.unwrap();

        let foo_union_dev = foo_dev.to_enum();

        if let union_foo_dev::FooOnlyReserveA2AndA3Union::A2(a2_dev) = foo_union_dev {
            assert_eq!(a2_0_7_3.as_slice(), a2_dev.as_slice());
        } else {
            panic!("foo_union_dev should be A2");
        }
    }

    #[test]
    fn test_decode_0_7_3_generated_deprecated_rust_bytes_by_current_version() {
        let a0_0_7_3 = union_foo_0_7_3::A0::new_builder()
            .nth0(Byte::from(133))
            .build();

        let foo_0_7_3 = union_foo_0_7_3::Foo::new_builder().set(a0_0_7_3).build();
        let foo_0_7_3_slice = foo_0_7_3.as_slice();

        let foo_dev_result = union_foo_dev::FooOnlyReserveA2AndA3::from_slice(foo_0_7_3_slice);
        assert!(foo_dev_result.is_err());
    }
}
