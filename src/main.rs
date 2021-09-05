use std::fs::File;

use ac_ffmpeg::{Error, codec::{Decoder, audio::AudioDecoder}, format::{
        demuxer::{Demuxer, DemuxerWithStreamInfo},
        io::IO,
    }};

/// Open a given input file.
fn open_input(path: &str) -> Result<DemuxerWithStreamInfo<File>, Error> {
    let input = File::open(path)
        .map_err(|err| Error::new(format!("unable to open input file {}: {}", path, err)))?;

    let io = IO::from_seekable_read_stream(input);

    Demuxer::builder()
        .build(io)?
        .find_stream_info(None)
        .map_err(|(_, err)| err)
}

/// Decode all video frames from the first video stream and print their
/// presentation timestamps.
fn print_audio_frame_info(input: &str) -> Result<(), Error> {
    let mut demuxer = open_input(input)?;

    let (stream_index, params) = demuxer
        .streams()
        .iter()
        .map(|stream| stream.codec_parameters())
        .enumerate()
        .find(|(_, params)| params.is_audio_codec())
        .ok_or_else(|| Error::new("no video stream"))?;

    let params = params.as_audio_codec_parameters().unwrap();

    let mut decoder = AudioDecoder::from_codec_parameters(params)?.build()?;

    // process data
    while let Some(packet) = demuxer.take()? {
        if packet.stream_index() != stream_index {
            continue;
        }

        decoder.push(packet)?;

        while let Some(frame) = decoder.take()? {
            println!("{}", frame.pts().as_f32().unwrap_or(0f32));
        }
    }

    decoder.flush()?;

    while let Some(frame) = decoder.take()? {
        println!("{}", frame.pts().as_f32().unwrap_or(0f32));
    }

    Ok(())
}

fn main() {
    /*
    let matches = App::new("decoding_audio")
        .arg(
            Arg::with_name("input")
                .required(true)
                .takes_value(true)
                .value_name("INPUT")
                .help("Input file"),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();
    */

    if let Err(err) = print_audio_frame_info("testing.m4a") {
        eprintln!("ERROR: {}", err);
    }
}
