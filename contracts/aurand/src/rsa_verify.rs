use crate::utils::sha512_hash;
use crate::error::ContractError;
use rsa::{BigUint,PaddingScheme,RsaPublicKey,PublicKey};
use sha2::Sha512;

/// random org MODULUS for generate public key
const MODULUS: &str = concat!("ecedc74162e74f30828ffab0a08e2f8ff4fddb7ef07bbe2bc1c256db0e12bb320a565027e72",
"85a25c69e429769987c2642ddda53c1b56daee7df197b85d78f921f9a12460cde254e84965d9022a3cf0db1ee55124089d",
"992c827b3c47888692524f2275fa7e606312bb7562b8c8f01e47ab3de4a226e4a8866056e67541f26881b9acad3eb88a68",
"220dd786dd70dc398e320f34bbdf86cda9150d6216b76839f0bf1aee6f23217d6b41976cba9d72836de30a27d356bbbdb7",
"57b2fe04615e12f60c3eaf22791549ef271abca7925c4a22f46be0cc28eecb618124e5ece353b97f4ed59ea1b1722eaeab",
"26e5120af44a83444d816726c49592bcb24cfb4eee58798dd160e1098705411fcdf71640c9318f82db0ef447327e5422ba",
"1f900ee0fbded67ff2109d9ce195987e0e021bde38d70f9d06a89b1dedc774a23259bb319fe812d267c836299389dcab41",
"d6efe76781d541474fe99368a77984c7b3226abef04838d1cc68386b27f11daf293ad13aa3ca5ed1dee556edd74c70bd90",
"be6a6775ea95de92c7db49d99436a038d33e53c885818c2dd78485799852b8670c2869389ad6bec6ff7a1e0cdfcb1651c7",
"0141397db01bd6464adb4826b3971640f98e4a38f109dcd211f068ca14dc1b77c064f589372e76e8712a7713cd81543d60",
"8b8cd177d32d0610a519cfffc62f12e56ac5868f25fac67e742abf8ae5582d39065");
/// random org EXPONENT for generate public key
const EXPONENT: &str = "010001";

/// verify random value receive from random org
pub fn verify_message(data: String, signature: String) -> Result<bool, ContractError> {
    let n: BigUint = BigUint::from_bytes_be(&hex::decode(MODULUS)
        .map_err(|_| ContractError::CustomError{val: String::from("Invalid HEX string format for MODULUS!")})?);
    let e: BigUint = BigUint::from_bytes_be(&hex::decode(EXPONENT)
        .map_err(|_| ContractError::CustomError{val: String::from("Invalid HEX string format for EXPONENT!")})?);

    // generate RSA public key from MODULUS and EXPONENT
    let rsa = RsaPublicKey::new(n,e).map_err(|_| ContractError::CustomError{val: String::from("Invalid RSA public key!")})?;

    let signature_bytes = base64::decode(signature)
        .map_err(|_| ContractError::CustomError{val: String::from("Invalid base64 string!")})?;
    let hashed_data = sha512_hash(data.as_bytes());

    // verify signature using RSA pkcs#1v15
    let padding = PaddingScheme::new_pkcs1v15_sign::<Sha512>();
    match rsa.verify(padding,&hashed_data,&signature_bytes) {
        Ok(_) => {
            return Ok(true);
        },
        Err(_) => {
            return Ok(false);
        }
    };
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn verify_work_with_ok(){
        let data = r#"{"method":"generateSignedIntegers","hashedApiKey":"oT3AdLMVZKajz0pgW/8Z+t5sGZkqQSOnAi1aB8Li0tXgWf8LolrgdQ1wn9sKx1ehxhUZmhwUIpAtM8QeRbn51Q==","n":6,"min":1,"max":6,"replacement":true,"base":10,"data":[6,1,4,4,3,6],"completionTime":"2014-06-03 17:15:13Z","serialNumber":79924}"#.to_string();
        let signature = "XWTB2PiGutI86GYDNIEiYvbTkAC1PQO3U2A/Depb2m2W4zUF81UFjTthCNmvPYFdnrBlGMgS7mo1rNUKfkVU9M0Yv0fPkjVaYoDo3ADOw1DGtENtU+Em+Clhowz+FQEhfUTLOBTfruYpnb1CSjbovo8AzjHF0pb+0F8awVMZPuHEhjE8oHJcQInVXmkLq/IR5WNcM0E0ygRQto37NE9CIFDst+5WAN7UmlqYTNil+iqmzjj92vTDlHr+Gh3bhgxb+aR9rabpaGQni2MlyXH0kGCrbAdryvCzUTZ/SxXY6MWfmNFODzvibcO2j//GFm/Z8uyVuyeAt5GNO0QQipWvv8eauALAW87JDLw8vgYcbFapHIAsWOyrhD9tMMmaejKzc+leMwvs0BSy6I8jwLBy6MlcPUHO3i4JFs+0qstKtqaVzmUGm+fnfJPZLySHBBazrX0tMpn36FyiE3wn8XYncOJM1ylUNdT9j2A+xp3ZuoMkr4+Fv6Flh444B+eeqEdZTlgSmXDh7VFoCrcks4QO2KJ0ajzltNv42fO5KdizOPg1fV1totJivzsxA4i0+RnhpPO9tdT4iYjBcuNSdh9nYDtcn7cizODaCr6Y+oOzfIktBok19YjebgMd+AbDhkVmHmPEsaOuL62eqdmCobwPJUjVtM8cgccQqfkfek30uK4=".to_string();
        
        assert_eq!(verify_message(data,signature).unwrap(),true);
    }

    #[test]
    fn verify_work_with_err(){
        let data = r#"some random data"#.to_string();
        let signature = "XWTB2PiGutI86GYDNIEiYvbTkAC1PQO3U2A/Depb2m2W4zUF81UFjTthCNmvPYFdnrBlGMgS7mo1rNUKfkVU9M0Yv0fPkjVaYoDo3ADOw1DGtENtU+Em+Clhowz+FQEhfUTLOBTfruYpnb1CSjbovo8AzjHF0pb+0F8awVMZPuHEhjE8oHJcQInVXmkLq/IR5WNcM0E0ygRQto37NE9CIFDst+5WAN7UmlqYTNil+iqmzjj92vTDlHr+Gh3bhgxb+aR9rabpaGQni2MlyXH0kGCrbAdryvCzUTZ/SxXY6MWfmNFODzvibcO2j//GFm/Z8uyVuyeAt5GNO0QQipWvv8eauALAW87JDLw8vgYcbFapHIAsWOyrhD9tMMmaejKzc+leMwvs0BSy6I8jwLBy6MlcPUHO3i4JFs+0qstKtqaVzmUGm+fnfJPZLySHBBazrX0tMpn36FyiE3wn8XYncOJM1ylUNdT9j2A+xp3ZuoMkr4+Fv6Flh444B+eeqEdZTlgSmXDh7VFoCrcks4QO2KJ0ajzltNv42fO5KdizOPg1fV1totJivzsxA4i0+RnhpPO9tdT4iYjBcuNSdh9nYDtcn7cizODaCr6Y+oOzfIktBok19YjebgMd+AbDhkVmHmPEsaOuL62eqdmCobwPJUjVtM8cgccQqfkfek30uK4=".to_string();
        
        assert_eq!(verify_message(data,signature).unwrap(),false);
    }

    #[test]
    fn verify_fail_with_invalid_base64_string() {
        let data = r#"some random data"#.to_string();
        let signature = "@#*B2PiGutI86GYDNIEiYvbTkAC1PQO3U2A/Depb2m2W4zUF81UFjTthCNmvPYFdnrBlGMgS7mo1rNUKfkVU9M0Yv0fPkjVaYoDo3ADOw1DGtENtU+Em+Clhowz+FQEhfUTLOBTfruYpnb1CSjbovo8AzjHF0pb+0F8awVMZPuHEhjE8oHJcQInVXmkLq/IR5WNcM0E0ygRQto37NE9CIFDst+5WAN7UmlqYTNil+iqmzjj92vTDlHr+Gh3bhgxb+aR9rabpaGQni2MlyXH0kGCrbAdryvCzUTZ/SxXY6MWfmNFODzvibcO2j//GFm/Z8uyVuyeAt5GNO0QQipWvv8eauALAW87JDLw8vgYcbFapHIAsWOyrhD9tMMmaejKzc+leMwvs0BSy6I8jwLBy6MlcPUHO3i4JFs+0qstKtqaVzmUGm+fnfJPZLySHBBazrX0tMpn36FyiE3wn8XYncOJM1ylUNdT9j2A+xp3ZuoMkr4+Fv6Flh444B+eeqEdZTlgSmXDh7VFoCrcks4QO2KJ0ajzltNv42fO5KdizOPg1fV1totJivzsxA4i0+RnhpPO9tdT4iYjBcuNSdh9nYDtcn7cizODaCr6Y+oOzfIktBok19YjebgMd+AbDhkVmHmPEsaOuL62eqdmCobwPJUjVtM8cgccQqfkfek30uK4=".to_string();
        
        let result = verify_message(data,signature).unwrap_err();

        match result {
            ContractError::CustomError{val: v} => {assert_eq!(v, String::from("Invalid base64 string!"))},
            _ => panic!(),
        }
    }
}