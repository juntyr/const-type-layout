use alloc::fmt;
use core::{marker::PhantomData, ops::Deref};

use crate::{Field, TypeLayoutGraph, TypeLayoutInfo, Variant};

use serde::{
    de::{Error, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

pub fn serialize<
    'a,
    F: Deref<Target = [Field<'a>]> + Serialize,
    V: Deref<Target = [Variant<'a, F>]> + Serialize,
    I: Deref<Target = TypeLayoutInfo<'a, F, V>> + Serialize,
    S: Serializer,
>(
    tys: &[Option<I>; TypeLayoutGraph::CAPACITY],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut len = 0;

    let mut it = tys.iter();

    while let Some(Some(_)) = it.next() {
        len += 1;
    }

    let mut seq = serializer.serialize_seq(Some(len))?;

    let mut it = tys.iter();

    while let Some(Some(ty)) = it.next() {
        seq.serialize_element(ty)?;
    }

    seq.end()
}

pub fn deserialize<
    'de,
    F: Deref<Target = [Field<'de>]> + Deserialize<'de>,
    V: Deref<Target = [Variant<'de, F>]> + Deserialize<'de>,
    I: Deref<Target = TypeLayoutInfo<'de, F, V>> + Deserialize<'de>,
    D: Deserializer<'de>,
>(
    deserializer: D,
) -> Result<[Option<I>; TypeLayoutGraph::CAPACITY], D::Error> {
    deserializer.deserialize_seq(TypeListVisitor {
        marker: PhantomData,
    })
}

struct TypeListVisitor<
    'de,
    F: Deref<Target = [Field<'de>]>,
    V: Deref<Target = [Variant<'de, F>]>,
    I: Deref<Target = TypeLayoutInfo<'de, F, V>>,
> {
    marker: PhantomData<(F, V, I)>,
}

impl<
        'de,
        F: Deref<Target = [Field<'de>]> + Deserialize<'de>,
        V: Deref<Target = [Variant<'de, F>]> + Deserialize<'de>,
        I: Deref<Target = TypeLayoutInfo<'de, F, V>> + Deserialize<'de>,
    > Visitor<'de> for TypeListVisitor<'de, F, V, I>
{
    type Value = [Option<I>; TypeLayoutGraph::CAPACITY];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut tys = [(); TypeLayoutGraph::CAPACITY].map(|_| None);

        let mut i = 0;

        while let Some(ty) = seq.next_element()? {
            let Some(slot) = tys.get_mut(i) else {
                return Err(Error::custom(
                    "TypeLayoutGraph is not large enough for this complex type"
                ))
            };

            *slot = Some(ty);

            i += 1;
        }

        Ok(tys)
    }
}
