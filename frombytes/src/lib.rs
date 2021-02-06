use std::iter::Iterator;
use std::result::Result;
use std::thread::current;

pub enum ParsingError {
    InvalidByte,
    NotEnoughBytes,
}

pub trait FromBytes {
    type Output;

    fn from_bytes<T: Iterator<Item = u8>>(bytes: T) -> Result<(Self::Output, T), ParsingError>;
}

pub struct Chunk<T: Iterator> {
    inner: T,
    size: usize,
    current: usize,
}

impl<T: Iterator> Chunk<T> {
    pub fn new(iterator: T, size: usize) -> Self {
        Chunk {
            inner: iterator,
            size,
            current: 0,
        }
    }
}

impl<T: Iterator> Iterator for Chunk<T> {
    type Item = <T as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let value = if self.current < self.size {
            self.inner.next()
        } else if self.current == self.size {
            None
        } else {
            self.inner.next()
        };
        self.current += 1;
        value
    }
}

//impl FromBytes for u8 {
//    type Output = Self;
//
//    fn from_bytes<T: Iterator<Item = u8>>(bytes: T) -> Result<(Self::Output, T), ParsingError>;
//}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(PartialEq, Debug)]
    struct Cut {
        flags: u16,
        value: u32,
    }

    #[test]
    fn chunks() {
        let mut value = vec![0, 1, 2, 3, 4, 5, 6];
        let mut iter = value.into_iter();
        let c = Chunk::new(iter, 2);

        assert_eq!(vec![0, 1], c.copied().collect::<Vec<i32>>());
        assert_eq!(vec![2, 3, 4, 5, 6], c.collect::<Vec<i32>>());
    }

    //impl FromBytes for Cut {
    //    type Output = Self;
    //    fn from_bytes<T: Iterator<Item = u8>>(bytes: T) -> Result<(Self::Output, T), ParsingError> {
    //        let mut peekable_iterator = bytes.peekable();

    //        let it = bytes.;
    //        let size = std::mem::size_of::<u16>();
    //        let flags: u16 = {
    //           for byte, count in it.enumerate() {

    //            }
    //        };
    //        let value: u32 = 0xAABBCCDD;
    //        for byte in 0..std::mem::size_of::<u16>() {
    //            println!("Foo");
    //            peekable_iterator.next();
    //        }
    //        bytes = peekable_iterator.into();
    //        Ok((Cut { flags, value }, bytes))
    //    }
    //}

    //#[test]
    //fn from_bytes_consume_iterator() {
    //    let input: Vec<u8> = vec![0x00, 0x01, 0xAA, 0xBB, 0xCC, 0xDD];
    //    let expected = Cut {
    //        flags: 0x00001,
    //        value: 0xAABBCCDD,
    //    };
    //    let result = Cut::from_bytes(input.into_iter());
    //    assert!(result.is_ok());
    //    assert_eq!(expected, result.ok().unwrap().0);
    //}

    #[test]
    fn from_bytes_borrow_iterator() {
        //let input: Vec<u8> = vec![0x00, 0x01, 0xAA, 0xBB, 0xCC, 0xDD];
        assert_eq!(2 + 2, 4);
    }
}
