# Lazy Marshal
This is (yet another) serialization/deserialization crate. The idea behind it is to be lazy so it can work
effectively for communicating over sockets.

---

This can be used with data streams that implement [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html) by calling
`.bytes()` to produce a [`Bytes`](https://doc.rust-lang.org/std/io/struct.Bytes.html) struct which implements `Iterator`.

# Examples
You can marshal built in types:
```rs
use lazy_marshal::prelude::*;

let i: Vec<u8> = 260u32.marshal().collect();
assert!(i == vec![0, 0, 1, 4]);
```

And unmarshal them
```rs
use lazy_marshal::prelude::*;

let mut i = "Hello, World!".marshal();
let decoded = String::unmarshal(&mut i).unwrap();

assert_eq!("Hello, World!".to_string(), decoded);
```
> ![NOTE]
> `&str` will marshall to the same thing as `String` and will unmarshal to `String`

Unmarshalling only consumes the bytes needed from the iterator to produce the desired object.
```rs
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
```

You can use the `#[derive(Marshal, Unmarshal)]` derive macros for automatic implementation on custom structs/enums
```rs
use lazy_marshal::prelude::*;

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

fn main() {
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

    let mut hmap: HashMap<Salesman, Vec<Deal>> = HashMap::new();
    for deal in deals {
        hmap.entry(deal.salesman.clone())
            .or_insert(Vec::new())
            .push(deal)
    }

    // Produces the iterator that can be unmarshalled later
    let mut iter = hmap.clone().marshal();

    // let new_hmap: HashMap<Salesman, Vec<Deal>> = UnMarshal::unmarshal(&mut iter).unwrap();
    let new_hmap = HashMap::<Salesman, Vec<Deal>>::unmarshal(&mut iter).unwrap();

    assert!(hmap == new_hmap);
    assert!(iter.next() == None);
}
```