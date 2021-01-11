use nom::bytes::complete::{tag, take};
use nom::number::complete::{
    be_f32, be_i32, be_u16, be_u32, be_u8, le_f32, le_i32, le_u16, le_u32, le_u8,
};
use nom::sequence::pair;
use nom::{Finish, IResult, ToUsize};

use crate::CineonError;

impl From<nom::Err<nom::error::Error<&[u8]>>> for CineonError {
    fn from(_error: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::ParserError
    }
}

pub(crate) trait ReadBytes {
    fn read_u8<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u8>;
    fn read_u16<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u16>;
    fn read_u32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u32>;
    fn read_i32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], i32>;
    fn read_f32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], f32>;
    fn read_u8_pair<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (u8, u8)>;
    fn read_f32_pair<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (f32, f32)>;
}

pub(crate) trait BoxClone: ReadBytes {
    fn box_clone(&self) -> Box<dyn ReadBytes>;
}

pub(crate) fn check_magick(input: &[u8], magick: u32) -> Result<(&[u8], &[u8]), CineonError> {
    tag(magick.to_be_bytes())(input)
        .finish()
        .map_err(|_: nom::error::Error<&[u8]>| CineonError::NotCineonImage)
}

pub(crate) fn read_bytes<C: ToUsize>(
    count: C,
) -> impl Fn(&[u8]) -> Result<(&[u8], &[u8]), CineonError> {
    let c = count.to_usize();
    move |input: &[u8]| {
        take(c)(input)
            .finish() // Using complete and not streaming functions, so acceptable
            .map_err(|_: nom::error::Error<&[u8]>| CineonError::ParserError)
    }
}

pub(crate) fn read_string<C: ToUsize>(
    count: C,
) -> impl Fn(&[u8]) -> Result<(&[u8], String), CineonError> {
    let c = count.to_usize();
    move |input: &[u8]| {
        match take(c)(input)
            .finish() // Using complete and not streaming functions, so acceptable
            .map_err(|_: nom::error::Error<&[u8]>| CineonError::ParserError)
        {
            Ok((i, v)) => match std::str::from_utf8(v) {
                Ok(str_value) => Ok((i, str_value.trim_matches(char::from(0)).to_owned())),
                Err(_) => Err(CineonError::StringConversion),
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct LittleEndian;

impl ReadBytes for LittleEndian {
    fn read_u8<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u8> {
        le_u8(input)
    }
    fn read_u16<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u16> {
        le_u16(input)
    }
    fn read_u32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u32> {
        le_u32(input)
    }
    fn read_i32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], i32> {
        le_i32(input)
    }
    fn read_f32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], f32> {
        le_f32(input)
    }
    fn read_u8_pair<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (u8, u8)> {
        pair(le_u8, le_u8)(input)
    }
    fn read_f32_pair<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (f32, f32)> {
        pair(le_f32, le_f32)(input)
    }
}

impl BoxClone for LittleEndian {
    fn box_clone(&self) -> Box<dyn ReadBytes> {
        Box::new((*self).clone())
    }
}

#[derive(Default, Clone)]
pub(crate) struct BigEndian;

impl ReadBytes for BigEndian {
    fn read_u8<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u8> {
        be_u8(input)
    }
    fn read_u16<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u16> {
        be_u16(input)
    }
    fn read_u32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], u32> {
        be_u32(input)
    }
    fn read_i32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], i32> {
        be_i32(input)
    }
    fn read_f32<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], f32> {
        be_f32(input)
    }
    fn read_u8_pair<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (u8, u8)> {
        pair(be_u8, be_u8)(input)
    }
    fn read_f32_pair<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (f32, f32)> {
        pair(be_f32, be_f32)(input)
    }
}

impl BoxClone for BigEndian {
    fn box_clone(&self) -> Box<dyn ReadBytes> {
        Box::new((*self).clone())
    }
}

pub(crate) trait Output {}

impl Output for u8 {}
impl Output for u16 {}
impl Output for u32 {}
impl Output for i32 {}
impl Output for f32 {}
impl Output for (u8, u8) {}
impl Output for (f32, f32) {}

trait RunFunctions {
    fn run_single_closure<'b, F, V>(&self, func: F, input: &'b [u8]) -> IResult<&'b [u8], V>
    where
        F: for<'r, 'a> FnOnce(&'r (dyn ReadBytes + 'static), &'a [u8]) -> IResult<&'a [u8], V>,
        V: Output;
}

impl RunFunctions for Box<dyn ReadBytes> {
    fn run_single_closure<'b, F, V>(&self, func: F, input: &'b [u8]) -> IResult<&'b [u8], V>
    where
        F: for<'r, 'a> FnOnce(&'r (dyn ReadBytes + 'static), &'a [u8]) -> IResult<&'a [u8], V>,
        V: Output,
    {
        func(&**self, input)
    }
}

trait CloneBytes {
    fn clone_bytes(&self) -> Box<dyn ReadBytes>;
}

impl CloneBytes for Box<dyn BoxClone> {
    fn clone_bytes(&self) -> Box<dyn ReadBytes> {
        self.box_clone()
    }
}

pub(crate) struct Endian(Box<dyn BoxClone>);

impl Endian {
    pub(crate) fn new<T: BoxClone + 'static>(item: T) -> Self {
        Self(Box::new(item))
    }

    pub(crate) fn run<'b, F, V>(&self, func: F) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], V>
    where
        F: for<'r, 'a> FnOnce(&'r (dyn ReadBytes + 'static), &'a [u8]) -> IResult<&'a [u8], V>
            + Copy,
        V: Output,
    {
        let a = self.0.clone_bytes();
        move |input: &[u8]| a.run_single_closure(func, input)
    }
}
