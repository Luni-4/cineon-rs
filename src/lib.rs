mod header;
mod parser;

use header::{
    DataFormatInfo, FileInformation, FilmInfo, ImageChannel, ImageInfo, OriginationInfo,
    MAX_ELEMENTS,
};

use header::{Header, MAGIC_COOKIE};
use parser::ReadBytes;
use parser::{check_magick, read_bytes, read_string, BigEndian, Endian, LittleEndian};

/// Cineon Error
#[derive(Debug)]
pub enum CineonError {
    /// The file is not a Cineon image
    NotCineonImage,
    /// An error occurred while parsing the file
    ParserError,
    /// String conversion error
    StringConversion,
    /// An error occurred while writing a new Cineon image
    OutputError,
}

/// Image Data
pub struct ImageData {
    /// Cineon Header
    pub header: Header,
    /// Pixels
    pub pixels: Vec<u8>,
}

/// Cineon Parser
#[derive(Default)]
pub struct Cineon;

impl Cineon {
    /// Checks whether the input is a cineon image.
    pub fn is_cineon(&self, input: &[u8]) -> bool {
        check_magick(input, MAGIC_COOKIE).is_ok()
    }

    /// Parses the header of a cineon image.
    pub fn parse_header(&self, input: &[u8]) -> Result<Header, CineonError> {
        self.parse_header_inner(input).map(|(_, v)| v)
    }

    /// Parses image data.
    pub fn parse_image(&self, input: &[u8]) -> Result<ImageData, CineonError> {
        let (i, header) = self.parse_header_inner(input)?;
        let image_height = header.image_info.channel[0].lines_per_element;

        let bytes_per_row = if header.image_info.number_of_elements == 1 {
            Self::bytes_per_row(
                1, // Monochrome image
                header.image_info.channel[0].bit_depth as u32,
                header.image_info.channel[0].pixels_per_line,
                true,
            )
        } else {
            Self::bytes_per_row(
                3, // Three planes image
                header.image_info.channel[0].bit_depth as u32,
                header.image_info.channel[0].pixels_per_line,
                true,
            )
        };

        let total_bytes = image_height * bytes_per_row;
        let (_, pixels) = read_bytes(total_bytes)(i)?;

        Ok(ImageData {
            header,
            pixels: pixels.to_owned(),
        })
    }

    fn bytes_per_row(samples_per_pixel: u32, bit_depth: u32, width: u32, pad: bool) -> u32 {
        match bit_depth {
            1 | 8 | 32 => 4 * ((samples_per_pixel * width * bit_depth + 31) / 32),
            10 => {
                if !pad {
                    4 * ((samples_per_pixel * width * bit_depth + 31) / 32)
                } else {
                    4 * (((32 * ((samples_per_pixel * width + 2) / 3)) + 31) / 32)
                }
            }
            12 => {
                if !pad {
                    4 * ((samples_per_pixel * width * bit_depth + 31) / 32)
                } else {
                    2 * (((16 * samples_per_pixel * width) + 15) / 16)
                }
            }
            16 => 2 * ((samples_per_pixel * width * bit_depth + 8) / 16),
            64 => 8 * ((samples_per_pixel * width * bit_depth + 63) / 64),
            _ => 4 * ((samples_per_pixel * width * bit_depth + 31) / 32),
        }
    }

    #[inline(always)]
    fn is_big_endian(magick: &[u8]) -> bool {
        magick[0] == 0x80 && magick[1] == 0x2a && magick[2] == 0x5f && magick[3] == 0xd7
    }

    fn parse_header_inner<'a>(&self, input: &'a [u8]) -> Result<(&'a [u8], Header), CineonError> {
        let (i, magick_number) = check_magick(input, MAGIC_COOKIE)?;
        let parser: Endian = if Self::is_big_endian(magick_number) {
            Endian::new(BigEndian::default())
        } else {
            Endian::new(LittleEndian::default())
        };
        let (i, file_info) = Self::parse_file_info(i, &parser)?;
        let (i, image_info) = Self::parse_image_info(i, &parser)?;
        let (i, data_format_info) = Self::parse_data_format_info(i, &parser)?;
        let (i, origination_info) = Self::parse_origination_info(i, &parser)?;
        let (i, film_info) = if file_info.image_offset > 2048 && file_info.user_size != 0 {
            let (i, film_info) = Self::parse_film_info(i, &parser)?;
            (i, Some(film_info))
        } else {
            // 1024 is the film info size
            let (i, _) = read_bytes(1024usize)(i)?;
            (i, None)
        };
        let (i, user_info) = if file_info.image_offset > 2048 && file_info.user_size != 0 {
            let (i, user_info) = read_bytes(file_info.user_size)(i)?;
            (i, Some(user_info.to_owned()))
        } else {
            (i, None)
        };
        let header = Header {
            file_info,
            image_info,
            data_format_info,
            origination_info,
            film_info,
            user_info,
        };

        Ok((i, header))
    }

    fn parse_file_info<'a>(
        input: &'a [u8],
        parser: &Endian,
    ) -> Result<(&'a [u8], FileInformation), CineonError> {
        let (i, image_offset) = parser.run(ReadBytes::read_u32)(input)?;
        let (i, generic_size) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, industry_size) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, user_size) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, file_size) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, version) = read_string(8u8)(i)?;
        let (i, filename) = read_string(100u8)(i)?;
        let (i, creation_date) = read_string(12u8)(i)?;
        let (i, creation_time) = read_string(12u8)(i)?;
        let (i, _) = read_bytes(36u8)(i)?;
        Ok((
            i,
            FileInformation {
                magic_number: MAGIC_COOKIE,
                image_offset,
                generic_size,
                industry_size,
                user_size,
                file_size,
                version,
                filename,
                creation_date,
                creation_time,
            },
        ))
    }

    fn parse_image_info<'a>(
        input: &'a [u8],
        parser: &Endian,
    ) -> Result<(&'a [u8], ImageInfo), CineonError> {
        let (i, image_orientation) = parser.run(ReadBytes::read_u8)(input)?;
        let (i, number_of_elements) = parser.run(ReadBytes::read_u8)(i)?;
        let (i, _) = read_bytes(2u8)(i)?;

        let mut channel: [ImageChannel; MAX_ELEMENTS] = [ImageChannel::default(); MAX_ELEMENTS];
        let mut state = i;
        for item in channel.iter_mut() {
            let (i, (designator_0, designator_1)) = parser.run(ReadBytes::read_u8_pair)(state)?;
            let (i, bit_depth) = parser.run(ReadBytes::read_u8)(i)?;
            let (i, _) = read_bytes(1u8)(i)?;
            let (i, pixels_per_line) = parser.run(ReadBytes::read_u32)(i)?;
            let (i, lines_per_element) = parser.run(ReadBytes::read_u32)(i)?;
            let (i, min_data) = parser.run(ReadBytes::read_f32)(i)?;
            let (i, min_quantity) = parser.run(ReadBytes::read_f32)(i)?;
            let (i, max_data) = parser.run(ReadBytes::read_f32)(i)?;
            let (i, max_quantity) = parser.run(ReadBytes::read_f32)(i)?;
            *item = ImageChannel {
                designator: [designator_0, designator_1],
                bit_depth,
                pixels_per_line,
                lines_per_element,
                min_data,
                min_quantity,
                max_data,
                max_quantity,
            };
            state = i;
        }
        let (i, (white_point_x, white_point_y)) = parser.run(ReadBytes::read_f32_pair)(state)?;
        let (i, (red_primary_x, red_primary_y)) = parser.run(ReadBytes::read_f32_pair)(i)?;
        let (i, (green_primary_x, green_primary_y)) = parser.run(ReadBytes::read_f32_pair)(i)?;
        let (i, (blue_primary_x, blue_primary_y)) = parser.run(ReadBytes::read_f32_pair)(i)?;
        let (i, label_text) = read_string(200u8)(i)?;
        let (i, _) = read_bytes(28u8)(i)?;
        Ok((
            i,
            ImageInfo {
                image_orientation: image_orientation.into(),
                number_of_elements,
                channel,
                white_point: [white_point_x, white_point_y],
                red_primary: [red_primary_x, red_primary_y],
                green_primary: [green_primary_x, green_primary_y],
                blue_primary: [blue_primary_x, blue_primary_y],
                label_text,
            },
        ))
    }

    fn parse_data_format_info<'a>(
        input: &'a [u8],
        parser: &Endian,
    ) -> Result<(&'a [u8], DataFormatInfo), CineonError> {
        let (i, interleave) = parser.run(ReadBytes::read_u8)(input)?;
        let (i, packing) = parser.run(ReadBytes::read_u8)(i)?;
        let (i, data_sign) = parser.run(ReadBytes::read_u8)(i)?;
        let (i, image_sense) = parser.run(ReadBytes::read_u8)(i)?;
        let (i, line_padding) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, channel_padding) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, _) = read_bytes(20u8)(i)?;
        Ok((
            i,
            DataFormatInfo {
                interleave: interleave.into(),
                packing: packing.into(),
                data_sign: data_sign != 0,
                image_sense: image_sense != 0,
                line_padding: Some(line_padding),
                channel_padding: Some(channel_padding),
            },
        ))
    }

    fn parse_origination_info<'a>(
        input: &'a [u8],
        parser: &Endian,
    ) -> Result<(&'a [u8], OriginationInfo), CineonError> {
        let (i, x_offset) = parser.run(ReadBytes::read_i32)(input)?;
        let (i, y_offset) = parser.run(ReadBytes::read_i32)(i)?;
        let (i, source_image_file_name) = read_string(100u8)(i)?;
        let (i, source_date) = read_string(12u8)(i)?;
        let (i, source_time) = read_string(12u8)(i)?;
        let (i, input_device) = read_string(64u8)(i)?;
        let (i, input_device_model_number) = read_string(32u8)(i)?;
        let (i, input_device_serial_number) = read_string(32u8)(i)?;
        let (i, x_device_pitch) = parser.run(ReadBytes::read_f32)(i)?;
        let (i, y_device_pitch) = parser.run(ReadBytes::read_f32)(i)?;
        let (i, gamma) = parser.run(ReadBytes::read_f32)(i)?;
        let (i, _) = read_bytes(40u8)(i)?;
        Ok((
            i,
            OriginationInfo {
                x_offset,
                y_offset,
                source_image_file_name,
                source_date,
                source_time,
                input_device,
                input_device_model_number,
                input_device_serial_number,
                x_device_pitch,
                y_device_pitch,
                gamma,
            },
        ))
    }

    fn parse_film_info<'a>(
        input: &'a [u8],
        parser: &Endian,
    ) -> Result<(&'a [u8], FilmInfo), CineonError> {
        let (i, film_manufacturing_id_code) = parser.run(ReadBytes::read_u8)(input)?;
        let (i, film_type) = parser.run(ReadBytes::read_u8)(i)?;
        let (i, perfs_offset) = parser.run(ReadBytes::read_u8)(i)?;
        let (i, _) = read_bytes(1u8)(i)?;
        let (i, prefix) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, count) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, format) = read_string(32u8)(i)?;
        let (i, frame_position) = parser.run(ReadBytes::read_u32)(i)?;
        let (i, frame_rate) = parser.run(ReadBytes::read_f32)(i)?;
        let (i, frame_id) = read_string(32u8)(i)?;
        let (i, slate_info) = read_string(200u8)(i)?;
        let (i, _) = read_bytes(740usize)(i)?;
        Ok((
            i,
            FilmInfo {
                film_manufacturing_id_code,
                film_type,
                perfs_offset,
                prefix,
                count,
                format,
                frame_position,
                frame_rate,
                frame_id,
                slate_info,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &'static [u8] = include_bytes!("../assets/data.cin");

    #[test]
    fn is_cineon() {
        assert_eq!(Cineon::default().is_cineon(DATA), true);
    }

    #[test]
    fn read_header() {
        assert!(Cineon::default().parse_header(DATA).is_ok());
    }

    #[test]
    fn read_image() {
        assert!(Cineon::default().parse_image(DATA).is_ok());
    }
}
