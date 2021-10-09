pub use mckerel_protocol_macros::{enum_impl, Packet};

macro_rules! packets_impl {
    ($name:ident {
        $($type:ident = $tag:literal),*
    }) => {
        pub enum $name {
            $($type($type)),*
        }

        impl<'de> crate::de::Deserialize<'de> for $name {
            type Value = Self;

            fn deserialize(input: &mut crate::de::ByteReader<'de>) -> crate::de::Result<Self> {
                let tag = <crate::varnum::VarInt as crate::de::Deserialize>::deserialize(input)?;
                match tag {
                    $($tag => {
                        let val = <$type as crate::de::Deserialize<'de>>::deserialize(input)?;
                        Ok(Self::$type(val))
                    }),*
                    _ => Err(crate::de::Error::BadPacketId)
                }
            }
        }
    }
}