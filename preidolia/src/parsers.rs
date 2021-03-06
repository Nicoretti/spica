use anyhow::{anyhow, Result};
use nom::*;
use std::io::Read;

fn parse_from_read<I: Read, O: Sized>(
    parser: &dyn Fn(&[u8]) -> nom::IResult<&[u8], O>,
    buffer: &mut Vec<u8>,
    input: &mut I,
) -> Result<O> {
    loop {
        match (parser)(&buffer) {
            Ok((_, value)) => return Ok(value),
            Err(e) => match e {
                Err::Incomplete(Needed::Size(size)) => {
                    let mut buf: Vec<u8> = vec![0; size.get()];
                    match input.read_exact(&mut buf) {
                        Ok(_) => {
                            buffer.append(&mut buf);
                        }
                        Err(e) => return Err(anyhow::Error::new(e)),
                    };
                }
                Err::Incomplete(Needed::Unknown) => {
                    let mut buf: Vec<u8> = vec![0; 1];
                    match input.read_exact(&mut buf) {
                        Ok(_) => {
                            buffer.append(&mut buf);
                        }
                        Err(e) => return Err(anyhow::Error::new(e)),
                    };
                }
                Err::Error(e) => {
                    return Err(anyhow!("Failed to parse, {:?}", e));
                }
                Err::Failure(e) => {
                    return Err(anyhow!("Failed to parse, {:?}", e));
                }
            },
        }
    }
}

pub struct Parser<'a, O: Sized> {
    parser: &'a dyn Fn(&[u8]) -> nom::IResult<&[u8], O>,
}

impl<'a, O: Sized> Parser<'a, O> {
    pub fn new(parser: &'a dyn Fn(&[u8]) -> nom::IResult<&[u8], O>) -> Self {
        Parser { parser }
    }

    fn parse<I: Read>(&mut self, input: &mut I) -> Result<O> {
        let mut buf: Vec<u8> = Vec::new();
        parse_from_read(self.parser, &mut buf, input)
    }
}

pub struct ParsingIterator<'a, R: Read, O: Sized> {
    input: R,
    parser: Parser<'a, O>,
}

impl<'a, R: Read, O: Sized> ParsingIterator<'a, R, O> {
    pub fn new(parser: Parser<'a, O>, input: R) -> Self {
        ParsingIterator { input, parser }
    }
}

impl<'a, R: Read, O> Iterator for ParsingIterator<'a, R, O> {
    type Item = Result<O>;

    fn next(&mut self) -> Option<Result<O>> {
        match self.parser.parse(&mut self.input) {
            Ok(value) => Some(Ok(value)),
            Err(e) => match e.root_cause().downcast_ref::<std::io::Error>() {
                Some(io_error) => match io_error.kind() {
                    std::io::ErrorKind::UnexpectedEof => None,
                    _ => Some(Err(e)),
                },
                _ => Some(Err(e)),
            }, //Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::number::streaming::be_u32;
    use nom::number::streaming::be_u8;

    #[derive(Debug, PartialEq)]
    struct Identifier {
        id1: u8,
        id2: u8,
        id3: u8,
        id4: u32,
        id5: u32,
    }

    fn parse_u32(input: &[u8]) -> nom::IResult<&[u8], u32> {
        do_parse!(input, value: be_u32 >> (value))
    }

    fn parse_identifier(input: &[u8]) -> nom::IResult<&[u8], Identifier> {
        do_parse!(
            input,
            id1: be_u8
                >> id2: be_u8
                >> id3: be_u8
                >> id4: be_u32
                >> id5: be_u32
                >> (Identifier {
                    id1,
                    id2,
                    id3,
                    id4,
                    id5
                })
        )
    }

    #[test]
    fn test_parse_u32() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02];

        let cursor = std::io::Cursor::new(input);
        let parser: Parser<u32> = Parser::new(&parse_u32);
        let mut iter = ParsingIterator::new(parser, cursor);

        {
            let item = iter.next();
            assert!(item.is_some());
            let result = item.unwrap();
            assert!(result.is_ok());
            let value = result.unwrap();
            assert_eq!(value, 1);
        }
        {
            let item = iter.next();
            assert!(item.is_some());
            let result = item.unwrap();
            assert!(result.is_ok());
            let value = result.unwrap();
            assert_eq!(value, 2);
        }
        {
            let item = iter.next();
            assert!(item.is_none());
        }
    }

    #[test]
    fn smoke_test() {
        let expected = Identifier {
            id1: 0x01,
            id2: 0x02,
            id3: 0x00,
            id4: 0xffaaffaa,
            id5: 0xffaaffaa,
        };
        let input: Vec<u8> = vec![
            0x01, // id1
            0x02, // id2
            0x00, // id3
            0xff, 0xaa, 0xff, 0xaa, // id4
            0xff, 0xaa, 0xff, 0xaa, // id5
        ];

        let mut cursor = std::io::Cursor::new(input);
        let mut parser: Parser<Identifier> = Parser::new(&parse_identifier);
        let r = parser.parse(&mut cursor);
        assert!(r.is_ok());
        assert_eq!(expected, r.unwrap())
    }
}
