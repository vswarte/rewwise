pub mod cstring {
    use std::ffi;
    use serde::{Serialize, Deserialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &ffi::CStr, s: S) -> Result<S::Ok, S::Error> {
        String::serialize(&v.to_string_lossy().to_string(), s)
    }
    
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<ffi::CString, D::Error> {
        Ok(ffi::CString::new(String::deserialize(d)?).unwrap())
    }
}

pub mod bytestring {
    use serde::{Serialize, Deserialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let string = String::from_utf8_lossy(v.as_slice());
        String::serialize(&string.as_ref().to_owned(), s)
    }
    
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let string = String::deserialize(d)?;
        Ok(string.as_bytes().to_vec())
    }
}

pub mod base64 {
    use base64::Engine;
    use serde::{Serialize, Deserialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        String::serialize(&base64::engine::general_purpose::STANDARD_NO_PAD.encode(v), s)
    }
    
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        Ok(base64::engine::general_purpose::STANDARD_NO_PAD.decode(String::deserialize(d)?)
            .unwrap())
    }
}
