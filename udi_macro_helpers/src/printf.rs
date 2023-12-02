
pub struct Error<'a> {
    pub pos: usize,
    pub kind: ErrorKind<'a>,
}
pub enum ErrorKind<'a> {
    UnexpectedEof,
    ExpectedDigit(u8),
    InvalidFragment(&'a [u8]),
    UnexpectedChar(&'static str, u8),
}
impl ::std::fmt::Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
        ErrorKind::UnexpectedEof => f.write_str("Unexpected end of string"),
        ErrorKind::ExpectedDigit(have) => write!(f, "Expected a digit, got {:?}", have as char),
        ErrorKind::InvalidFragment(_) => f.write_str("Invalid printf fragment"),
        ErrorKind::UnexpectedChar(exp, have) => write!(f, "Unexpected character, got {:?} expected {}", have as char, exp),
        }
    }
}
#[derive(Debug)]
pub enum FormatArg<'a> {
    StringData(&'a [u8]),
    
    Pointer(bool),
    String(PadKind, u32),
    BusAddr(bool),
    Char,
    Integer(PadKind, u32, Size, IntFormat),
    BitSet(BitsetParser<'a>),
}
#[derive(Debug)]
pub enum PadKind {
    LeadingZero,
    LeftPad,
    Unspec,
}
#[derive(Debug)]
pub enum Size {
    U32,
    U16,
    U8,
}
#[derive(Debug)]
pub enum IntFormat {
    UpperHex,
    LowerHex,
    Decimal,
    Unsigned,
}

#[derive(Debug)]
pub enum BitsetEnt<'a> {
    Single(u32, bool, &'a [u8]),
    Range(u32,u32, &'a [u8], RangeNamesParser<'a>),
}

#[derive(Debug)]
pub struct Parser<'a> {
    inner: ParserCommon<'a>,
}
impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Parser { inner: ParserCommon::new(input) }
    }
    pub fn next(&mut self) -> Result< Option<FormatArg<'a>>, Error<'a> > {
        let start = self.inner.pos;
        match self.next_c()
        {
        Err(_) => Ok(None),
        Ok(b'%') => {
            let mut c = self.next_c()?;
            if c == b'%' {
                return Ok(Some(FormatArg::StringData(&self.inner.input[start+1..][..1])));
            }

            let pad_kind = if c == b'0' {
                    // Leading zero pad requested
                    c = self.next_c()?;
                    PadKind::LeadingZero
                }
                else if c == b'-' {
                    // Left pad, not allowed with `0`
                    c = self.next_c()?;
                    PadKind::LeftPad
                }
                else {
                    PadKind::Unspec
                };
            // Width
            let width = if c.is_ascii_digit() {
                self.parse_num(&mut c)?
            }
            else {
                0
            };
    
            Ok(Some(match c {
                b'X' => FormatArg::Integer(pad_kind, width, Size::U32, IntFormat::UpperHex),
                b'x' => FormatArg::Integer(pad_kind, width, Size::U32, IntFormat::LowerHex),
                b'd' => FormatArg::Integer(pad_kind, width, Size::U32, IntFormat::Decimal),
                b'u' => FormatArg::Integer(pad_kind, width, Size::U32, IntFormat::Unsigned),
                b'h' => match self.next_c()?
                    {
                    b'X' => FormatArg::Integer(pad_kind, width, Size::U16, IntFormat::UpperHex),
                    b'x' => FormatArg::Integer(pad_kind, width, Size::U16, IntFormat::LowerHex),
                    b'd' => FormatArg::Integer(pad_kind, width, Size::U16, IntFormat::Decimal),
                    b'u' => FormatArg::Integer(pad_kind, width, Size::U16, IntFormat::Unsigned),
                    _ => return Err(self.inner.error(2, |i| ErrorKind::InvalidFragment(i))),
                    },
                b'b' => match self.next_c()?
                    {
                    b'X' => FormatArg::Integer(pad_kind, width, Size::U8, IntFormat::UpperHex),
                    b'x' => FormatArg::Integer(pad_kind, width, Size::U8, IntFormat::LowerHex),
                    b'd' => FormatArg::Integer(pad_kind, width, Size::U8, IntFormat::Decimal),
                    b'u' => FormatArg::Integer(pad_kind, width, Size::U8, IntFormat::Unsigned),
                    _ => return Err(self.inner.error(2, |i| ErrorKind::InvalidFragment(i))),
                    },
                b'p' => FormatArg::Pointer(false),
                b'P' => FormatArg::Pointer(true ),
                b'a' => FormatArg::BusAddr(false),
                b'A' => FormatArg::BusAddr(true ),
                b'c' => FormatArg::Char,
                b's' => FormatArg::String(pad_kind, width),
                b'<' => {
                    let inner_start = self.inner.pos;
                    self.inner.consume_to(b'>');
                    let inner_end = self.inner.pos;
                    c = self.next_c()?;
                    if c != b'>' {
                        unreachable!();
                    }
                    FormatArg::BitSet( BitsetParser::new( &self.inner.input[inner_start..inner_end]) )
                    },
                _ => return Err(self.inner.error(1, |i| ErrorKind::InvalidFragment(i))),
                }))
            },
        Ok(_) => {
            self.inner.consume_to(b'%');
            Ok(Some( FormatArg::StringData(&self.inner.input[start..self.inner.pos]) ))
            },
        }
    }
    
    fn next_c(&mut self) -> Result<u8,Error<'a>> {
        self.inner.next_c()
    }
    fn parse_num(&mut self, c: &mut u8) -> Result<u32,Error<'a>> {
        self.inner.parse_num(c)
    }
}

#[derive(Debug)]
pub struct BitsetParser<'a> {
    inner: ParserCommon<'a>
}
impl<'a> BitsetParser<'a> {
    pub fn new(s: &'a [u8]) -> Self {
        Self { inner: ParserCommon::new(s) }
    }
    // Comma-separated list of:
    // - [~]BitNum=Name String
    // - Start-End=Name String{:Value=Name}
    pub fn next(&mut self) -> Result<Option<BitsetEnt>,Error> {
        let mut c = match self.next_c()
            {
            Err(_) => return Ok(None),
            Ok(v) => v,
            };

        // A leading comma is valid
        if c == b',' {
            c = self.next_c()?;
        }

        enum Ty {
            Single(u32, bool),
            Range(u32,u32),
        }
        

        let ty = if c == b'~' {
            // Single inverted bit
            c = self.next_c()?;
            let bitnum = self.parse_num(&mut c)?;
            Ty::Single(bitnum, true)
        }
        else {
            let start_pos = self.inner.pos - 1;
            let bitnum = self.parse_num(&mut c)?;
            if c != b'-' {
                // Single positive bit
                Ty::Single(bitnum, false)
            }
            else {
                // Bit range
                c = self.next_c()?;
                let end = self.parse_num(&mut c)?;
                if !(bitnum <= end) {
                    return Err(self.inner.error(self.inner.pos - start_pos, |i| ErrorKind::InvalidFragment(i)));
                }
                Ty::Range(bitnum, end)
            }
        };

        match c {
        b'=' => {},
        _ => return Err(self.inner.error(1, |_| ErrorKind::UnexpectedChar("`=`", c))),
        }
        c = self.next_c()?;
        let _ = c;

        match ty {
        Ty::Single(idx, inv) => {
            let start = self.inner.pos - 1;
            self.inner.consume_to(b',');
            let name = &self.inner.input[start..self.inner.pos];
            let _ = self.next_c();  // Consume the `,` (ignoring a potential EOF error)
            Ok(Some(BitsetEnt::Single(idx,inv,name)))
        }
        Ty::Range(start, end) => {
            // Consume the name
            let name = {
                let s = self.inner.pos - 1;
                while self.inner.pos < self.inner.input.len() && self.inner.input[self.inner.pos] != b',' && self.inner.input[self.inner.pos] != b':' {
                    self.inner.pos += 1;
                }
                &self.inner.input[s..self.inner.pos]
                };
            let tail = if let Ok(b':') = self.next_c()
                {
                    let start = self.inner.pos - 1;
                    self.inner.consume_to(b',');
                    &self.inner.input[start..self.inner.pos]
                }
                else {
                    &self.inner.input[self.inner.pos..self.inner.pos]
                };

            Ok(Some(BitsetEnt::Range(start, end, name, RangeNamesParser::new(tail))))
        },
        }
    }

    fn next_c(&mut self) -> Result<u8,Error<'a>> {
        self.inner.next_c()
    }
    fn parse_num(&mut self, c: &mut u8) -> Result<u32,Error<'a>> {
        self.inner.parse_num(c)
    }
}

#[derive(Debug)]
pub struct RangeNamesParser<'a> {
    inner: ParserCommon<'a>,
}
impl<'a> RangeNamesParser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            inner: ParserCommon::new(input)
        }
    }
    pub fn next(&mut self) -> Result<Option<(u32, &'a [u8])>,Error> {
        match self.inner.next_c() {
        Err(_) => return Ok(None),
        Ok(b':') => {},
        Ok(_) => return Err(self.inner.error(1, |i| ErrorKind::UnexpectedChar("`:`", i[0]))),
        }

        let mut c = self.inner.next_c()?;
        let val = self.inner.parse_num(&mut c)?;
        match c {
        b'=' => {},
        _ => return Err(self.inner.error(1, |_| ErrorKind::UnexpectedChar("`=`", c))),
        }
        let s = self.inner.pos;
        self.inner.consume_to(b':');
        let name = &self.inner.input[s..self.inner.pos];
        Ok(Some( (val, name) ))
    }
}

#[derive(Debug)]
struct ParserCommon<'a> {
    pos: usize,
    input: &'a [u8],
}
impl<'a> ParserCommon<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        ParserCommon { pos: 0, input }
    }

    fn consume_to(&mut self, ch: u8) {
        while self.pos < self.input.len() && self.input[self.pos] != ch {
            self.pos += 1;
        }
    }
    fn error(&self, ofs: usize, cb: impl FnOnce(&'a [u8])->ErrorKind<'a>) -> Error<'a> {
        Error { pos: self.pos, kind: cb(&self.input[self.pos - ofs..self.pos]) }
    }

    fn next_c(&mut self) -> Result<u8,Error<'a>> {
        let rv = match self.input.get(self.pos).copied()
            {
            Some(rv) => rv,
            None => return Err(Error { pos: self.pos, kind: ErrorKind::UnexpectedEof }),
            };
        self.pos += 1;
        Ok(rv)
    }
    fn parse_num(&mut self, c: &mut u8) -> Result<u32,Error<'a>> {
        let mut rv = 0;
        if !c.is_ascii_digit() {
            return Err(Error { pos: self.pos, kind: ErrorKind::ExpectedDigit(*c) });
        }
        while let Some(d) = (*c as char).to_digit(10) {
            rv *= 10;
            rv += d;
            *c = self.next_c()?;
        }
        Ok( rv )
    }
}