use fankor::prelude::*;

#[derive(Debug, Eq, PartialEq, EnumDiscriminants, FankorSerialize, FankorDeserialize)]
#[repr(u8)]
enum X {
    A,
    #[deprecated]
    B,

    #[discriminant = 5]
    C,
    #[deprecated]
    D,

    #[discriminant = 20]
    E(u64),
    #[deprecated]
    F,

    #[discriminant = 120]
    G {
        a: u64,
        b: u64,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use fankor::prelude::borsh::{BorshDeserialize, BorshSerialize};

    #[test]
    fn text_serialize_x() {
        let a = X::A;
        let b = X::B;
        let c = X::C;
        let d = X::D;
        let e = X::E(100);
        let f = X::F;
        let g = X::G { a: 100, b: 200 };

        let a = a.try_to_vec().unwrap();
        let b = b.try_to_vec().unwrap();
        let c = c.try_to_vec().unwrap();
        let d = d.try_to_vec().unwrap();
        let e = e.try_to_vec().unwrap();
        let f = f.try_to_vec().unwrap();
        let g = g.try_to_vec().unwrap();

        assert_eq!(a, vec![0]);
        assert_eq!(b, vec![1]);
        assert_eq!(c, vec![5]);
        assert_eq!(d, vec![6]);
        assert_eq!(e, vec![20, 100, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(f, vec![21]);
        assert_eq!(
            g,
            vec![120, 100, 0, 0, 0, 0, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_deserialize_x() {
        let a = X::A;
        let b = X::B;
        let c = X::C;
        let d = X::D;
        let e = X::E(100);
        let f = X::F;
        let g = X::G { a: 100, b: 200 };

        let a = a.try_to_vec().unwrap();
        let b = b.try_to_vec().unwrap();
        let c = c.try_to_vec().unwrap();
        let d = d.try_to_vec().unwrap();
        let e = e.try_to_vec().unwrap();
        let f = f.try_to_vec().unwrap();
        let g = g.try_to_vec().unwrap();

        assert_eq!(X::try_from_slice(&a).unwrap(), X::A);
        assert_eq!(X::try_from_slice(&b).unwrap(), X::B);
        assert_eq!(X::try_from_slice(&c).unwrap(), X::C);
        assert_eq!(X::try_from_slice(&d).unwrap(), X::D);
        assert_eq!(X::try_from_slice(&e).unwrap(), X::E(100));
        assert_eq!(X::try_from_slice(&f).unwrap(), X::F);
        assert_eq!(X::try_from_slice(&g).unwrap(), X::G { a: 100, b: 200 });
    }
}
