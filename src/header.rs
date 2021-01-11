/// Cineon format version V4.5

/// Maximum number of image elements
pub const MAX_ELEMENTS: usize = 8;

/// Maximum number of components per image element
//const MAX_COMPONENTS: usize = 8;

/// Magic Cookie value
pub const MAGIC_COOKIE: u32 = 0x802A5FD7;

/// File Information
#[derive(Default, Debug)]
pub struct FileInformation {
    /// Indicates start of Cineon image file and it is used to
    /// determine byte order.
    pub magic_number: u32,
    /// Offset to image data (in bytes)
    pub image_offset: u32,
    /// Generic Header length (in bytes)
    pub generic_size: u32,
    /// Industry Header length (in bytes)
    pub industry_size: u32,
    /// User defined header length (in bytes)
    pub user_size: u32,
    /// Total file size (in bytes)
    pub file_size: u32,
    /// Version number of header format
    pub version: String, // 8 bytes
    /// Filename
    pub filename: String, // 100 bytes
    /// Create date
    pub creation_date: String, // 12 bytes
    /// Create time
    pub creation_time: String, // 12 bytes
                               /*/// Reserved
                               reserved: [u8; 36],*/
}

/// Component interleaving method
#[derive(Debug)]
pub enum Interleave {
    /// Pixel interleave (rgbrgbrgb...)
    Pixel,
    /// Line interleave (rrr.ggg.bbb.rrr.ggg.bbb.)
    Line,
    /// Channel interleave (rrr..ggg..bbb..)
    Channel,
    /// Undefined interleave
    Undefined,
}

impl From<u8> for Interleave {
    #[inline(always)]
    fn from(x: u8) -> Self {
        match x {
            0 => Self::Pixel,
            1 => Self::Line,
            2 => Self::Channel,
            _ => Self::Undefined,
        }
    }
}

/// Component data packing method
#[derive(Debug)]
pub enum Packing {
    /// Use all bits (tight packing)
    Packed,
    /// Byte (8-bit) boundary, left justified
    ByteLeft,
    /// Byte (8-bit) boundary, right justified
    ByteRight,
    /// Word (16-bit) boundary, left justified
    WordLeft,
    /// Word (16-bit) boundary, right justified
    WordRight,
    /// Longword (32-bit) boundary, left justified
    LongWordLeft,
    /// Longword (32-bit) boundary, right justified
    LongWordRight,
    /// Pack as many fields as possible per cell, only one otherwise
    PackAsManyAsPossible,
    /// Undefined data packing
    Undefined,
}

impl From<u8> for Packing {
    #[inline(always)]
    fn from(x: u8) -> Self {
        match x {
            0 => Self::Packed,
            1 => Self::ByteLeft,
            2 => Self::ByteRight,
            3 => Self::WordLeft,
            4 => Self::WordRight,
            5 => Self::LongWordLeft,
            6 => Self::LongWordRight,
            7 => Self::PackAsManyAsPossible,
            _ => Self::Undefined,
        }
    }
}

/// Data Format Information
#[derive(Debug)]
pub struct DataFormatInfo {
    /// Data interleave
    pub interleave: Interleave,
    /// Packing
    pub packing: Packing,
    /// Data sign (0 = unsigned, 1 = signed)
    pub data_sign: bool, // 1 byte
    /// Image sense (0 = positive image, 1 = negative image)
    pub image_sense: bool, // 1 byte
    /// Line Padding
    pub line_padding: Option<u32>,
    /// Channel Padding
    pub channel_padding: Option<u32>,
    /*/// Reserved
    reserved: [u8; 20],*/
}

impl Default for DataFormatInfo {
    fn default() -> Self {
        Self {
            interleave: Interleave::Undefined,
            packing: Packing::Undefined,
            data_sign: false,
            image_sense: false,
            line_padding: None,
            channel_padding: None,
        }
    }
}

/// Image Channel
#[derive(Default, Debug, Clone, Copy)]
pub struct ImageChannel {
    /// Channel descriptor
    pub designator: [u8; 2],
    /// Bits per pixel
    pub bit_depth: u8,
    /*/// Reserved
    reserved: u8,*/
    /// Pixels per line
    pub pixels_per_line: u32,
    /// Lines per element
    pub lines_per_element: u32,

    /// Reference min data code value
    pub min_data: f32,
    /// Reference min quantity represented
    pub min_quantity: f32,
    /// Reference max data code value
    pub max_data: f32,
    /// Reference max quantity represented
    pub max_quantity: f32,
}

/// Image Orientation Code
#[derive(Debug)]
pub enum Orientation {
    /// Oriented top to bottom, left to right
    TopToBottomLeftToRight,
    /// Oriented top to bottom, right to left
    TopToBottomRightToLeft,
    /// Oriented bottom to top, left to right
    BottomToTopLeftToRight,
    /// Oriented bottom to top, right to left
    BottomToTopRightToLeft,
    /// Oriented left to right, top to bottom
    LeftToRightTopToBottom,
    /// Oriented right to left, top to bottom
    RightToLeftTopToBottom,
    /// Oriented left to right, bottom to top
    LeftToRightBottomToTop,
    /// Oriented right to left, bottom to top
    RightToLeftBottomToTop,
    /// Undefined Orientation
    Undefined,
}

impl From<u8> for Orientation {
    #[inline(always)]
    fn from(x: u8) -> Self {
        match x {
            0 => Self::TopToBottomLeftToRight,
            1 => Self::TopToBottomRightToLeft,
            2 => Self::BottomToTopLeftToRight,
            3 => Self::BottomToTopRightToLeft,
            4 => Self::LeftToRightTopToBottom,
            5 => Self::RightToLeftTopToBottom,
            6 => Self::LeftToRightBottomToTop,
            7 => Self::RightToLeftBottomToTop,
            _ => Self::Undefined,
        }
    }
}

/// Image Information
#[derive(Debug)]
pub struct ImageInfo {
    /// Image orientation
    pub image_orientation: Orientation,
    /// Number of elements (1-8)
    pub number_of_elements: u8,
    ///// Reserved (word alignment)
    //pub reserved: [u8; 2],
    /// Image element data structures
    pub channel: [ImageChannel; MAX_ELEMENTS],
    /// White point (x,y, pair)
    pub white_point: [f32; 2],
    /// Red primary chromaticity (x, y pair)
    pub red_primary: [f32; 2],
    /// Green primary chromaticity (x, y pair)
    pub green_primary: [f32; 2],
    /// Blue primary chromaticity (x, y pair)
    pub blue_primary: [f32; 2],

    /// Label text
    pub label_text: String, // 200 bytes
                            ///// Reserved
                            //pub reserved: [u8; 28],
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            image_orientation: Orientation::Undefined,
            number_of_elements: 1,
            channel: [ImageChannel::default(); MAX_ELEMENTS],
            white_point: [0.; 2],
            red_primary: [0.; 2],
            green_primary: [0.; 2],
            blue_primary: [0.; 2],
            label_text: String::new(),
        }
    }
}

/// Origination Information
#[derive(Default, Debug)]
pub struct OriginationInfo {
    /// X offset
    pub x_offset: i32,
    /// Y offset
    pub y_offset: i32,

    /// Source image filename
    pub source_image_file_name: String, // 100 bytes
    /// Source date
    pub source_date: String, // 12 bytes
    /// Source time
    pub source_time: String, // 12 bytes
    /// Input device name
    pub input_device: String, // 64 bytes
    /// Input device model number
    pub input_device_model_number: String, // 32 bytes
    /// Input device serial number
    pub input_device_serial_number: String, // 32 bytes

    /// X device pitch (samples/mm)
    pub x_device_pitch: f32,
    /// Y device pitch (samples/mm)
    pub y_device_pitch: f32,
    /// Gamma
    pub gamma: f32,
    /*/// Reserved
    reserved: [u8; 40],*/
}

/// Motion Picture and Television Industry Specific Information
#[derive(Default, Debug)]
pub struct FilmInfo {
    /// Film edge code manufacturing ID code
    pub film_manufacturing_id_code: u8,
    /// Film edge code type
    pub film_type: u8,
    /// Film edge code offset in perfs
    pub perfs_offset: u8,
    /*/// Reserved (word alignment)
    unused1: u8,*/
    /// Film edge code prefix
    pub prefix: u32,
    /// Film edge code count
    pub count: u32,

    /// Format string, e.g. Academy
    pub format: String, // 32 bytes

    /// Frame position in sequence
    pub frame_position: u32,

    /// Frame rate of original (frame / sec)
    pub frame_rate: f32,

    /// Frame identification, e.g. keyframe
    pub frame_id: String, // 32 bytes
    /// Slate information
    pub slate_info: String, // 200 bytes
                            /*/// Reserved
                            reserved1: [u8; 740],*/
}

/// Generic File and Image Header Information
#[derive(Default, Debug)]
pub struct Header {
    /// File Information
    pub file_info: FileInformation,

    /// Image Information
    pub image_info: ImageInfo,

    /// Data Format Information
    pub data_format_info: DataFormatInfo,

    /// Origination Information
    pub origination_info: OriginationInfo,

    /// Film Information.
    ///
    /// Cannot be present in the Cineon image.
    pub film_info: Option<FilmInfo>,

    /// User info
    ///
    /// In Cineon, this field contains a postage stamp image, whose size is
    /// 96x64x3 channels, so 18 KBytes for 8 bit mode, oriented correctly
    /// for display.
    pub user_info: Option<Vec<u8>>,
}
