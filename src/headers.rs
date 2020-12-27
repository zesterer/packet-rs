#[doc(hidden)]
pub use ::bitfield::bitfield;
#[doc(hidden)]
pub use ::bitfield::BitRange;
#[doc(hidden)]
pub use paste::paste;
#[doc(hidden)]
pub use std::any::Any;

pub trait Header: Send {
    fn name(&self) -> &str;
    fn len(&self) -> usize;
    fn show(&self);
    fn as_slice(&self) -> &[u8];
    fn clone(&self) -> Box<dyn Header>;
    fn to_owned(self) -> Box<dyn Header>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Declares a header
///
/// This macro will generate get and set methods for each field of the header.
/// In addition, each header will also come with the [Header](headers/trait.Header.html) trait implemented.
/// Finally, a few associate functions are provided for ease of use.
///
/// The macro's syntax is composed of 3 sections
/// * A header name followed by the total size in bytes
/// * This is followed by a comma separated field list with each field specifying the name, start and end bit location
/// * Lastly, an optional vector is allowed to specify the default values of the header fields. The size of the vector has to match the header length
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate rscapy;
/// # use rscapy::headers::*;
/// # fn main() {}
/// make_header!(
/// Vlan 4
/// (
///     pcp: 0-2,
///     cfi: 3-3,
///     vid: 4-15,
///     etype: 16-31
/// )
/// vec![0x0, 0xa, 0x8, 0x0]
/// );
/// ```
#[macro_export]
macro_rules! make_header {
    (
        $name: ident $size: literal
        ( $($field: ident: $start: literal-$end: literal),* )
        $x:expr
    ) => {
        paste! {
            bitfield! {
                pub struct $name(MSB0 [u8]);
                u64;
                $(
                    pub $field, [<set_ $field>]: $end, $start;
                )*
            }
            impl $name<Vec<u8>> {
                pub fn new() -> $name<Vec<u8>> {
                    $name($x)
                }
            }
            impl<T: AsMut<[u8]> + AsRef<[u8]>> $name<T> {
                pub fn bytes(&self, msb: usize, lsb: usize) -> Vec<u8> {
                    let bit_len = ::bitfield::size_of::<u8>() * 8;
                    assert_eq!((msb-lsb+1)%bit_len, 0);
                    let mut value: Vec<u8> = Vec::new();
                    for i in (lsb..=msb).step_by(bit_len) {
                        let v: u8 = self.bit_range(i + 7, i);
                        value.push(v);
                    }
                    value
                }
                pub fn set_bytes(&mut self, msb: usize, lsb: usize, value: &[u8]) {
                    let bit_len = ::bitfield::size_of::<u8>() * 8;
                    assert_eq!(value.len() * bit_len, msb-lsb+1);
                    let mut iter = 0;
                    for i in (lsb..=msb).step_by(bit_len) {
                        self.set_bit_range(i + 7, i, value[iter]);
                        iter += 1;
                    }
                }
                pub fn size() -> usize {
                    $size
                }
                pub fn len(&self) -> usize {
                    $size
                }
                pub fn name(&self) -> &str {
                    stringify!($name)
                }
                $(
                    pub fn [<$field _size>]() -> usize {
                        $end - $start + 1
                    }
                    pub fn [<$field _lsb>]() -> usize {
                        $start
                    }
                    pub fn [<$field _msb>]() -> usize {
                        $end
                    }
                )*
                pub fn show(&self) {
                    println!("#### {:16} {} {}", stringify!($name), "Size  ", "Data");
                    println!("-------------------------------------------");
                    $(
                    print!("{:20}: {:4} : ", stringify!($field), $end - $start + 1);
                    if (($end - $start + 1) <= 8) {
                        let x: u8 = self.bit_range($end, $start);
                        print!("{:02x}", x);
                    } else if (($end - $start + 1)%8 == 0){
                        let d = ($end - $start + 1)/8;
                        for i in ($start..(d*8 + $start)).step_by(8) {
                            let x: u8 = self.bit_range(i + 7, i);
                            print!("{:02x} ", x);
                        }
                    } else {
                        let d = ($end - $start + 1)/8;
                        let r = ($end - $start + 1)%8;
                        for i in ($start..(d*8 + $start)).step_by(8) {
                            let x: u8 = self.bit_range(i + 7, i);
                            print!("{:02x} ", x);
                        }
                        let x: u8 = self.bit_range($end, $end - r);
                        print!("{:02x}", x);
                    }
                    println!();
                    )*
                }
                pub fn clone(&self) -> $name<Vec<u8>> {
                    $name(Vec::from(self.0.as_ref()))
                }
                pub fn as_slice(&self) -> &[u8] {
                    self.0.as_ref()
                }
            }
            /*
            impl<'a> Into<&'a mut $name<Vec<u8>>> for &'a mut Box<dyn Header> {
                fn into(self) -> &'a mut $name<Vec<u8>> {
                    let b = match self.as_any_mut().downcast_mut::<$name<Vec<u8>>>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            impl<'a> Into<&'a $name<Vec<u8>>> for &'a Box<dyn Header> {
                fn into(self) -> &'a $name<Vec<u8>> {
                    let b = match self.as_any().downcast_ref::<$name<Vec<u8>>>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            */
            impl<'a> From<&'a Box<dyn Header>> for &'a $name<Vec<u8>> {
                fn from(s: &'a Box<dyn Header>) -> &'a $name<Vec<u8>> {
                    let b = match s.as_any().downcast_ref::<$name<Vec<u8>>>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            impl<'a> From<&'a mut Box<dyn Header>> for &'a mut $name<Vec<u8>> {
                fn from(s: &'a mut Box<dyn Header>) -> &'a mut $name<Vec<u8>> {
                    let b = match s.as_any_mut().downcast_mut::<$name<Vec<u8>>>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            impl Header for $name<Vec<u8>> {
                fn show(&self) {
                    self.show();
                }
                fn as_slice(&self) -> &[u8] {
                    self.as_slice()
                }
                fn clone(&self) -> Box<dyn Header> {
                    Box::new(self.clone())
                }
                fn to_owned(self) -> Box<dyn Header> {
                    Box::from(self)
                }
                fn name(&self) -> &str {
                    self.name()
                }
                fn len(&self) -> usize {
                    self.len()
                }
                fn as_any(&self) -> &dyn Any {
                    self
                }
                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }
            }
        }
    };
    (
        $name: ident $size: literal
        ( $($field: ident: $start: literal-$end: literal),* )
    ) => {
        make_header!(
            $name $size
            (
                $(
                    $field: $start-$end
                ),*
            )
            vec![0; $size]
        );
    };
}

#[cfg(feature = "python-module")]
use pyo3::prelude::*;

#[cfg(not(feature = "python-module"))]
extern crate pyo3_nullify;
#[cfg(not(feature = "python-module"))]
use pyo3_nullify::*;

#[macro_export]
macro_rules! make_header1 {
    (
        $name: ident $size: literal
        ( $($field: ident: $start: literal-$end: literal),* )
        $x:expr
    ) => {
        paste! {
            #[pyclass]
            pub struct $name {
                inner: Vec<u8>
            }
            impl ::bitfield::BitRange<u64> for $name {
                fn bit_range(&self, msb: usize, lsb: usize) -> u64 {
                    let bit_len = ::bitfield::size_of::<u8>() * 8;
                    let value_bit_len = ::bitfield::size_of::<u64>() * 8;
                    let mut value: u64 = 0;
                    for i in lsb..=msb {
                        value <<= 1;
                        value |= ((self.inner[i / bit_len] >> (bit_len - i % bit_len - 1)) & 1) as u64;
                    }
                    value << (value_bit_len - (msb - lsb + 1)) >> (value_bit_len - (msb - lsb + 1))
                }
                fn set_bit_range(&mut self, msb: usize, lsb: usize, value: u64) {
                    let bit_len = ::bitfield::size_of::<u8>() * 8;
                    let mut value = value;
                    for i in (lsb..=msb).rev() {
                        self.inner[i / bit_len] &= !(1 << (bit_len - i % bit_len - 1));
                        self.inner[i / bit_len] |= ((value & 1) as u8) << (bit_len - i % bit_len - 1);
                        value >>= 1;
                    }
                }
            }
            #[pymethods]
            impl $name {
                #[new]
                pub fn new() -> $name {
                    $name{ inner: $x}
                }
                $(
                pub fn $field(&self) -> u64 {
                    use ::bitfield::BitRange;
                    let raw_value: u64 = self.bit_range($end, $start);
                    ::bitfield::Into::into(raw_value)
                }
                pub fn [<set_ $field>](&mut self, value: u64) {
                    use ::bitfield::BitRange;
                    self.set_bit_range($end, $start, ::bitfield::Into::<u64>::into(value));
                }
                )*
                pub fn bytes(&self, msb: usize, lsb: usize) -> Vec<u8> {
                    let bit_len = ::bitfield::size_of::<u8>() * 8;
                    assert_eq!((msb-lsb+1)%bit_len, 0);
                    let mut value: Vec<u8> = Vec::new();
                    for i in (lsb..=msb).step_by(bit_len) {
                        let v: u8 = self.bit_range(i + 7, i) as u8;
                        value.push(v);
                    }
                    value
                }
                pub fn set_bytes(&mut self, msb: usize, lsb: usize, value: &[u8]) {
                    let bit_len = ::bitfield::size_of::<u8>() * 8;
                    assert_eq!(value.len() * bit_len, msb-lsb+1);
                    let mut iter = 0;
                    for i in (lsb..=msb).step_by(bit_len) {
                        self.set_bit_range(i + 7, i, value[iter] as u64);
                        iter += 1;
                    }
                }
                #[staticmethod]
                pub fn size() -> usize {
                    $size
                }
                pub fn len(&self) -> usize {
                    $size
                }
                pub fn name(&self) -> &str {
                    stringify!($name)
                }
                $(
                    #[staticmethod]
                    pub fn [<$field _size>]() -> usize {
                        $end - $start + 1
                    }
                    #[staticmethod]
                    pub fn [<$field _lsb>]() -> usize {
                        $start
                    }
                    #[staticmethod]
                    pub fn [<$field _msb>]() -> usize {
                        $end
                    }
                )*
                pub fn show(&self) -> () {
                    println!("#### {:16} {} {}", stringify!($name), "Size  ", "Data");
                    println!("-------------------------------------------");
                    $(
                    print!("{:20}: {:4} : ", stringify!($field), $end - $start + 1);
                    if (($end - $start + 1) <= 8) {
                        let x: u8 = self.bit_range($end, $start) as u8;
                        print!("{:02x}", x);
                    } else if (($end - $start + 1)%8 == 0){
                        let d = ($end - $start + 1)/8;
                        for i in ($start..(d*8 + $start)).step_by(8) {
                            let x: u8 = self.bit_range(i + 7, i) as u8;
                            print!("{:02x} ", x);
                        }
                    } else {
                        let d = ($end - $start + 1)/8;
                        let r = ($end - $start + 1)%8;
                        for i in ($start..(d*8 + $start)).step_by(8) {
                            let x: u8 = self.bit_range(i + 7, i) as u8;
                            print!("{:02x} ", x);
                        }
                        let x: u8 = self.bit_range($end, $end - r) as u8;
                        print!("{:02x}", x);
                    }
                    println!();
                    )*
                }
                pub fn clone(&self) -> $name {
                    $name{ inner: self.inner.clone() }
                }
                pub fn as_slice(&self) -> &[u8] {
                    self.inner.as_ref()
                }
            }
            /*
            impl<'a> Into<&'a mut $name<Vec<u8>>> for &'a mut Box<dyn Header> {
                fn into(self) -> &'a mut $name<Vec<u8>> {
                    let b = match self.as_any_mut().downcast_mut::<$name<Vec<u8>>>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            impl<'a> Into<&'a $name<Vec<u8>>> for &'a Box<dyn Header> {
                fn into(self) -> &'a $name<Vec<u8>> {
                    let b = match self.as_any().downcast_ref::<$name<Vec<u8>>>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            */
            impl<'a> From<&'a Box<dyn Header>> for &'a $name {
                fn from(s: &'a Box<dyn Header>) -> &'a $name {
                    let b = match s.as_any().downcast_ref::<$name>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            impl<'a> From<&'a mut Box<dyn Header>> for &'a mut $name {
                fn from(s: &'a mut Box<dyn Header>) -> &'a mut $name {
                    let b = match s.as_any_mut().downcast_mut::<$name>() {
                        Some(b) => b,
                        None => panic!("Header is not a {}", stringify!($name)),
                    };
                    b
                }
            }
            impl Header for $name {
                fn show(&self) {
                    self.show();
                }
                fn as_slice(&self) -> &[u8] {
                    self.as_slice()
                }
                fn clone(&self) -> Box<dyn Header> {
                    Box::new(self.clone())
                }
                fn to_owned(self) -> Box<dyn Header> {
                    Box::from(self)
                }
                fn name(&self) -> &str {
                    self.name()
                }
                fn len(&self) -> usize {
                    self.len()
                }
                fn as_any(&self) -> &dyn Any {
                    self
                }
                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }
            }
        }
    };
    (
        $name: ident $size: literal
        ( $($field: ident: $start: literal-$end: literal),* )
    ) => {
        make_header1!(
            $name $size
            (
                $(
                    $field: $start-$end
                ),*
            )
            vec![0; $size]
        );
    };
}

// ethernet header
make_header!(
Ethernet 14
(
    dst: 0-47,
    src: 48-95,
    etype: 96-111
)
vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5,
     0x6, 0x7, 0x8, 0x9, 0xa, 0xb,
     0x08, 0x00]
);

// vlan header
make_header!(
Vlan 4
(
    pcp: 0-2,
    cfi: 3-3,
    vid: 4-15,
    etype: 16-31
)
vec![0x0, 0xa, 0x08, 0x00]
);

// ipv4 header
make_header!(
IPv4 20
(
    version: 0-3,
    ihl: 4-7,
    diffserv: 8-15,
    total_len: 16-31,
    identification: 32-47,
    flags: 48-50,
    frag_startset: 51-63,
    ttl: 64-71,
    protocol: 72-79,
    header_checksum: 80-95,
    src: 96-127,
    dst: 128-159
)
vec![0x45, 0x00, 0x00, 0x14, 0x00, 0x33, 0x40, 0xdd, 0x40, 0x06, 0xfa, 0xec,
     0xc0, 0xa8, 0x0, 0x1,
     0xc0, 0xa8, 0x0, 0x2]
);

// ipv6 header
make_header!(
IPv6 40
(
    version: 0-3,
    traffic_class: 4-11,
    flow_label: 12-31,
    payload_len: 32-47,
    next_hdr: 48-55,
    hop_limit: 56-63,
    src: 64-191,
    dst: 192-319
)
vec![0x60, 0x00, 0x00, 0x00, 0x00, 0x2e, 0x06, 0x40,
     0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34,
     0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x35]
);

// tcp header
make_header!(
TCP 20
(
    src: 0-15,
    dst: 16-31,
    seq_no: 32-63,
    ack_no: 64-95,
    data_startset: 96-99,
    res: 100-103,
    flags: 104-111,
    window: 112-127,
    checksum: 128-143,
    urgent_ptr: 144-159
)
vec![0x04, 0xd2 , 0x00, 0x50, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
     0x50, 0x02, 0x20, 0x00, 0x0d, 0x2c, 0x0, 0x0]
);

// udp header
make_header!(
UDP 8
(
    src: 0-15,
    dst: 16-31,
    length: 32-47,
    checksum: 48-63
)
vec![0x04, 0xd2 , 0x00, 0x50, 0x0, 0x0, 0x0, 0x0]
);

// vxlan header
make_header!(
Vxlan 8
(
    flags: 0-7,
    reserved: 8-31,
    vni: 32-55,
    reserved2: 56-63
)
vec![0x8, 0x0 , 0x0, 0x0, 0x0, 0x07, 0xd0, 0x0]
);

make_header1!(
Tester 40
(
    bit1: 0-0,
    bit2: 1-2,
    bit3: 3-5,
    bit4: 6-9,
    bit5: 10-14,
    bit6: 15-20,
    bit7: 21-27,
    bit8: 28-35,
    bit9: 36-44,
    bit10: 45-47,
    byte1: 48-55,
    byte2: 56-71,
    byte3: 72-95,
    byte4: 66-127,
    byte8: 128-191,
    byte16: 192-319
)
vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
     0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0xf0, 0xe0, 0xd0, 0xc0,
     0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34, 0x45, 0x67,
     0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x35,
    ]
);

#[test]
fn test_header_get() {
    let test = Tester::new();
    assert_eq!(test.bit1(), 1);
    assert_eq!(test.bit2(), 3);
    assert_eq!(test.bit3(), 7);
    assert_eq!(test.bit4(), 15);
    assert_eq!(test.bit5(), 31);
    assert_eq!(test.bit6(), 63);
    assert_eq!(test.bit7(), 127);
    assert_eq!(test.bit8(), 255);
    assert_eq!(test.bit9(), 511);
    assert_eq!(test.bit10(), 7);
    assert_eq!(test.byte1(), 0x20);
    assert_eq!(test.byte2(), 0x010d);
    assert_eq!(test.byte3(), 0xb885a3);
    assert_eq!(test.byte4() as u32, 0xf0e0d0c0 as u32);
    assert_eq!(test.byte8(), 0x8a2e037073344567);
    let a = vec![
        0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73,
        0x35,
    ];
    let b = test.bytes(Tester::byte16_msb(), Tester::byte16_lsb());
    let b = b.as_slice();
    assert_eq!(a.iter().zip(b).filter(|&(a, b)| a == b).count(), 16);
}
#[test]
fn test_header_set() {
    let mut test = Tester::new();
    test.set_bit1(0);
    assert_eq!(test.bit1(), 0);
    test.set_bit2(2);
    assert_eq!(test.bit2(), 2);
    test.set_bit3(3);
    assert_eq!(test.bit3(), 3);
    test.set_bit4(4);
    assert_eq!(test.bit4(), 4);
    test.set_bit5(5);
    assert_eq!(test.bit5(), 5);
    test.set_bit6(6);
    assert_eq!(test.bit6(), 6);
    test.set_bit7(7);
    assert_eq!(test.bit7(), 7);
    test.set_bit8(8);
    assert_eq!(test.bit8(), 8);
    test.set_bit9(9);
    assert_eq!(test.bit9(), 9);
    test.set_bit10(3);
    assert_eq!(test.bit10(), 3);
    test.set_byte1(1);
    assert_eq!(test.byte1(), 1);
    test.set_byte1(0xFF);
    assert_eq!(test.byte1() as u8, 255);
    test.set_byte2(0xFFFF);
    assert_eq!(test.byte2() as u16, 0xFFFF);
    test.set_byte3(0xFFFFFF);
    assert_eq!(test.byte3() as u32, 0xFFFFFF);
    test.set_byte4(0xFFFFFFFF);
    assert_eq!(test.byte4() as u32, 0xFFFFFFFF);
    test.set_byte8(8);
    assert_eq!(test.byte8(), 8);
    test.set_byte8(0xFFFFFFFFFFFFFFFF);
    assert_eq!(test.byte8(), 0xFFFFFFFFFFFFFFFF);
    let a = vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10,
    ];
    test.set_bytes(Tester::byte16_msb(), Tester::byte16_lsb(), &a);
    let b = test.bytes(Tester::byte16_msb(), Tester::byte16_lsb());
    let b = b.as_slice();
    assert_eq!(a.iter().zip(b).filter(|&(a, b)| a == b).count(), 16);
}
