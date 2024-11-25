use std::collections::HashMap;

use crate::prelude::*;

#[test]
fn test_bool() {
    let t = true;
    let f = false;
    assert!(t == bool::unmarshal(&mut t.marshal()).unwrap());
    assert!(f == bool::unmarshal(&mut f.marshal()).unwrap());

    let mut v = [21u8, 23, 64].into_iter();

    assert!(bool::unmarshal(&mut v).is_err());
    let v = v.collect::<Vec<_>>();
    assert!(v[0] == 23);
    assert!(v[1] == 64);
}

#[test]
fn test_u128() {
    let n: u128 = 619818613546816;
    assert!(n == u128::unmarshal(&mut n.marshal()).unwrap());
}

#[test]
fn test_isize() {
    let n: isize = 619818613546816;
    assert!(n == isize::unmarshal(&mut n.marshal()).unwrap());
}

#[test]
fn test_f32() {
    let f = 69.420;
    assert!(f == f32::unmarshal(&mut f.marshal()).unwrap())
}

#[test]
fn test_f64() {
    let f = 69.420;
    assert!(f == f64::unmarshal(&mut f.marshal()).unwrap())
}

#[test]
fn test_str() {
    let s = "This is a test string 😁";

    let mut d = s.marshal();
    let decoded = String::unmarshal(&mut d).unwrap();
    assert_eq!(decoded, s);
}

#[test]
fn test_vec() {
    let d = vec![234, 5345, 45, 32456, 3243];
    let len = d.len();

    let marsh = d.clone().marshal().collect::<Vec<_>>();

    let new = Vec::<i32>::unmarshal(&mut marsh.into_iter()).unwrap();

    assert!(len == new.len());
    for i in 0..len {
        assert!(d[i] == new[i]);
    }
}

#[test]
fn test_option() {
    let d = Some(format!("Tesing"));
    let mut m = d.clone().marshal();
    let unm = Option::<String>::unmarshal(&mut m).unwrap();

    assert!(d == unm);

    let d: Option<Vec<Option<bool>>> = None;
    let mut m = d.marshal();
    let unm = Option::<Vec<Option<bool>>>::unmarshal(&mut m).unwrap();

    assert!(unm.is_none(), "failed to unmarshal None");

    let mut iter = [0, 1, 2, 3].into_iter();
    assert!(Option::<i32>::unmarshal(&mut iter).unwrap().is_none());
    assert!(iter.collect::<Vec<u8>>() == vec![1, 2, 3]);
}

#[derive(Debug, Clone, Marshal, UnMarshal, PartialEq, Eq, Hash)]
struct Salesman {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Clone, Marshal, UnMarshal, PartialEq, Eq)]
struct Deal {
    id: u32,
    name: String,
    salesman: Salesman,
}

#[test]
fn test_hashmap() {
    let s1 = Salesman {
        id: 1,
        name: "John Smith".to_string(),
        email: "abc@company.com".to_string(),
    };
    let s2 = Salesman {
        id: 2,
        name: "Mary Jane".to_string(),
        email: "mary@company.com".to_string(),
    };

    let deals = vec![
        Deal {
            id: 1,
            name: "Corp 1".to_string(),
            salesman: s1.clone(),
        },
        Deal {
            id: 2,
            name: "Corp 2".to_string(),
            salesman: s1.clone(),
        },
        Deal {
            id: 3,
            name: "Corp 3".to_string(),
            salesman: s2.clone(),
        },
        Deal {
            id: 4,
            name: "Corp 4".to_string(),
            salesman: s2.clone(),
        },
    ];

    let mut hmap = HashMap::new();
    for deal in deals {
        hmap.entry(deal.salesman.clone())
            .or_insert(Vec::new())
            .push(deal)
    }

    let mut iter = hmap.clone().marshal();

    let new_hmap = HashMap::<Salesman, Vec<Deal>>::unmarshal(&mut iter).unwrap();

    assert!(hmap == new_hmap);
    assert!(iter.next() == None);
}
