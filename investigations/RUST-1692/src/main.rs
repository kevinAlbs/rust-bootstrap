use bson;
use bson::spec::ElementType;

fn main() {
    // Deserialize signed integers:
    assert_eq!(bson::to_bson(&123i16).unwrap().element_type(), ElementType::Int32);
    assert_eq!(bson::to_bson(&123i32).unwrap().element_type(), ElementType::Int32);
    assert_eq!(bson::to_bson(&123i64).unwrap().element_type(), ElementType::Int64);

    // Deserialize unsigned integers:
    assert_eq!(bson::to_bson(&123u16).unwrap().element_type(), ElementType::Int32);
    assert_eq!(bson::to_bson(&123u32).unwrap().element_type(), ElementType::Int64); // I expect this is OK. u32::MAX cannot fit in Int32.
    assert_eq!(bson::to_bson(&123u64).unwrap().element_type(), ElementType::Int64);
}
