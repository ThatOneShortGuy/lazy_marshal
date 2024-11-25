use lazy_marshal::*;
use lazy_marshal_derive::*;

#[derive(Clone, Debug, Marshal, UnMarshal)]
pub struct Thing<T: Marshal + UnMarshal> {
    a: Vec<Option<String>>,
    b: String,
    c: T,
}

#[test]
fn test_struct() {
    let a = Thing {
        a: vec![Some("😉".to_string())],
        b: "Dancing cows".to_string(),
        c: Some(21),
    };
    let marshalled = a.clone().marshal().collect::<Vec<_>>();
    // panic!("{marshalled:#?}")

    let unmarshalled = Thing::<Option<i32>>::unmarshal(&mut marshalled.iter().cloned()).unwrap();

    assert!(a.a == unmarshalled.a);
    assert!(a.b == unmarshalled.b);
    assert!(a.c == unmarshalled.c);
}

#[derive(Debug, Clone, Marshal, UnMarshal)]
enum TestEnum {
    Part1(u32),
    Part2((String, u64)),
    NoPart,
}

// impl Marshal for TestEnum {
//     fn marshal(self) -> impl Iterator<Item = u8> {
//         match self {
//             Self::Part1(a) => MarshalIterator(Box::new(0u8.marshal().chain(a.marshal()))),
//             Self::Part2(v) => MarshalIterator(Box::new(1u8.marshal().chain(v.marshal()))),
//             Self::NoPart => MarshalIterator(Box::new(2u8.marshal())),
//         }
//     }
// }

// impl UnMarshal for TestEnum {
//     fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
//         let varient = u8::unmarshal(data)?;
//         Ok(match varient {
//             0 => Self::Part1(UnMarshal::unmarshal(data)?),
//             1 => Self::Part2(UnMarshal::unmarshal(data)?),
//             2 => Self::NoPart,
//             a => Err(MarshalError::InvalidData(format!(
//                 "Invalid enum varient: {a}"
//             )))?,
//         })
//     }
// }

#[test]
fn test_enum() {
    let e = TestEnum::NoPart;
    assert!(
        if let TestEnum::NoPart = TestEnum::unmarshal(&mut e.marshal()).unwrap() {
            true
        } else {
            false
        }
    );

    let tuple = (format!("test string"), 69);
    let e = TestEnum::Part2(tuple.clone());
    assert!(
        if let TestEnum::Part2(v) = TestEnum::unmarshal(&mut e.marshal()).unwrap() {
            v == tuple
        } else {
            false
        }
    );
}
