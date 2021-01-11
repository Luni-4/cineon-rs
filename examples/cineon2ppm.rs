use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use cineon::{Cineon, ImageData};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cineon2ppm")]
/// Input data
struct Opt {
    /// Input file
    #[structopt(short = "i", parse(from_os_str))]
    input: PathBuf,
    /// Output file
    #[structopt(short = "o", parse(from_os_str))]
    output: PathBuf,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    let file = File::open(opt.input).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut content = Vec::new();
    buf_reader.read_to_end(&mut content)?;

    let ImageData { header, pixels } = Cineon::default().parse_image(&content).unwrap();

    let width = header.image_info.channel[0].pixels_per_line;
    let height = header.image_info.channel[0].lines_per_element;
    let bit_depth = header.image_info.channel[0].bit_depth;

    let mut output = File::create(opt.output).unwrap();

    writeln!(&mut output, "P6").unwrap();
    writeln!(&mut output, "{} {}", width, height).unwrap();
    writeln!(&mut output, "{}", (1 << bit_depth) - 1).unwrap();

    output.write(&pixels).unwrap();

    Ok(())
}
