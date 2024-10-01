use std::{env::args, fs::File, i16, io::{Read, Write}, usize, vec};

use hound::{self, WavReader};
use image::{self, ImageReader};

const I24_MIN: i64 = -8388608;

fn show_help(path: &String) {
    let filename = std::path::Path::new(path).file_name().unwrap().to_str().unwrap();
    println!("Usage: {} [INPUT] [OPTIONS]", filename);
    println!("or drag'n'drop a supported file on {}", filename);
    println!("");
    println!("Supported formats: 16-bit .wav, 24-bit .wav, .png, .jpg, .jpeg, .gif, .bmp, .webp");
    println!("");
    println!("Converting a 16bit wav produces an image ign RGB565 mode");
    println!("");
    println!("Options:");
    println!("      -i, --input  [PATH]                   Input file");
    println!("      -o, --output [PATH]                   Output file");
    println!("      -g, --grayscale                       Process input in grayscale (Byte by byte)");
    println!("If converting from wav to image:");
    println!("      -d, --dimensions [WIDTH]x[HEIGHT]     Dimensions of output image");
    println!("      -g, --grayscale                       Outputs image in grayscale");
    println!("      -r, --rotate [VALUE]                  Rotates RGB");
    println!("If converting from image to wav:");
    println!("      -s, --stereo                          Outputs audio with 2 channels");
    println!("      -g, --grayscale                       Interpretes image as if it was in Grayscale");
    println!("      -16                                   Outputs audio in 16bit, assuming that image is in RGB565 mode");
    println!("");
}

fn process_path(args_struct: &mut Args, path: &String) -> bool {

    if File::open(path).is_err() {
        println!("Failed to open file!"); 
        let file = File::options().write(true).open(path);
        if file.is_err() {
            if File::create_new(path).is_err() {println!("file can't be created or written to"); return false};
            println!("file created");
        };
    };
    

    if args_struct.input.is_empty() {
        match path.split('.').last().unwrap() {
            //TODO: find a way to support all images available in the image crate
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => {println!("input is a picture"); args_struct.is_img = true;},
            "wav" => {println!("file is a wav"); args_struct.is_wav = true},
            _ => {println!("file not supported!"); return false;}
        }
    }

    
    println!("path: {}", path);
    true
}

fn process_args(args_struct: &mut Args) -> bool {
    let args: Vec<String> = args().collect();

    let mut arg = args.iter().peekable();

    let file_path = (*arg.peek().unwrap()).clone(); //:sob:

    arg.next();

    if arg.len() == 0 {show_help(&file_path); return false;};
    
    loop {
        let current_arg = match arg.peek() {
            Some(x) => *x,
            None => break
        };

        //drag n drop
        match current_arg.split('.').last().unwrap() {
            //TODO: find a way to support all images available in the image crate
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => {
                println!("input is a picture"); 
                args_struct.is_img = true;
                args_struct.input = current_arg.to_string();
                args_struct.output = current_arg.split_at(current_arg.rfind('.').unwrap()).0.to_string() + ".wav";
                return true;
            },
            "wav" => {
                println!("file is a wav"); 
                args_struct.is_wav = true;
                args_struct.input = current_arg.to_string();
                args_struct.output = current_arg.split_at(current_arg.rfind('.').unwrap()).0.to_string() + ".png";
                return true;
            },
            _ => ()
        }

        match arg.next().unwrap().as_str() {
            "-i" | "--input" => {
                println!("input found");

                let path = match arg.next() {
                    Some(x) => x,
                    None => {println!("-i requires an input file!"); return false;}
                };

                if !process_path(args_struct, path) {return false;};
                args_struct.input = path.to_string();
            },
            "-o" | "--output" => {
                println!("output found");
                
                let path = match arg.next() {
                    Some(x) => x,
                    None => {println!("-o requires an output path!"); return false;}
                };
                
                if !process_path(args_struct, path) {return false;};
                args_struct.output = path.to_string();
            },
            "-d" | "--dimensions"  => {
                println!("dimensions found");

                let res = match arg.next() {
                    Some(x) => x,
                    None => {println!("-d requires a resolution in <width>x<height> format!"); return false;}
                };

                let index = res.find('x').unwrap();
                let (x, mut y) = res.split_at(index);
                y = y.trim_start_matches('x');
                
                args_struct.dimensions[0] = x.trim().parse().expect("x is not a number!");
                args_struct.dimensions[1] = y.trim().parse().expect("y is not a number!");

                if args_struct.dimensions[0] <= 0 || args_struct.dimensions[1] <= 0 {println!("Invalid dimensions!"); return false;}

                println!("{}x{}", x, y);

            },
            "-s" | "--stereo" => {
                println!("using stereo");
                args_struct.stereo = true;
            },
            "-g" | "--grayscale" => {
                println!("using grayscale");
                args_struct.grayscale = true;
            }
            "-r" | "--rotate" => {
                println!("using rotate");
                
                let value: usize = match arg.next() {
                    Some(x) => x.trim().parse().expect("Invalid number!"),
                    None => {println!("-r requires a number!"); return false;}
                };
                let value = value.clamp(0, 3);

                args_struct.rotate = value;
            }
            "-sr" | "--sample-rate" => {
                println!("using sample rate");
                
                let value: u32 = match arg.next() {
                    Some(x) => x.trim().parse().expect("Invalid number!"),
                    None => {println!("-s requires a number!"); return false;}
                };

                if value <= 0 || value > 192000 {println!("Sample rate not supported!"); return false;}

                args_struct.samplerate = value;
            }
            "-16" => {
                println!("output as a 16 bit wav");

                args_struct.bit16 = true;
            }
            "-h" | "--help" => {
                //might be unnecessary
                if !args_struct.input.is_empty() {continue};

                show_help(&file_path);
                return false;
            }
            _ => {
                println!("Unrecognized argument: {}", current_arg);
            }
        }
    }
    true
}

fn wav_to_img(path: &String, args: &Args) {

    // TODO: add HSL mode for 24bit wav maybe?
    // take 9 bits for H, 8 for S, 7 for L

    let mut wav_as_file = File::open(&path).expect("Wav_as_file failed");
    let mut wav_bytes: Vec<u8> = vec![];
    wav_as_file.read_to_end(&mut wav_bytes).unwrap();
    println!("wav bytes: {}", wav_bytes.len());
    
    let mut wav = WavReader::open(&path).expect("Failed to open file");
    let mut wav_length: f32 = (wav.duration() - 0x50) as f32;
    // subtracting 0x50 cause wav.duration() returns the size of the WHOLE file and not the data chunk, 
    // which leads to invalid resolutions of images

    println!("wav length: {}", wav_bytes.len());
    
    if args.grayscale { wav_length *= 3 as f32; }
    wav_length *= wav.spec().channels as f32;
    
    let img_dimensions = wav_length.sqrt().ceil() as u32;
    let mut img_dimensions = (img_dimensions, img_dimensions);
    println!("img dimensions from wav: {:?}", img_dimensions);

    if !args.dimensions.contains(&0) {
        img_dimensions = (args.dimensions[0] as u32, args.dimensions[1] as u32);
    }

    match &wav_bytes[0x3C..0x40] {
        b"i2w " => {
                println!("i2w header found :D");
                img_dimensions.0 = u32::from_ne_bytes([wav_bytes[0x44], wav_bytes[0x45], wav_bytes[0x46], wav_bytes[0x47]]);
                img_dimensions.1 = u32::from_ne_bytes([wav_bytes[0x48], wav_bytes[0x49], wav_bytes[0x4a], wav_bytes[0x4b]]);
            },
        b"data" => println!("no i2w header :C"),
        _ => println!("unexpected header :C"),
    }

    let img_size = img_dimensions.0 * img_dimensions.1; 
    if img_size < wav.duration() {println!("Dimensions too small! Pic: {} < Samples: {}", img_size, wav.duration()); panic!();};

    // if args.grayscale {img_dimensions.0 *= 3; img_dimensions.1 *= 3;};
    let mut img = image::RgbImage::new(img_dimensions.0, img_dimensions.1);
    
    // let mut img_gray = image::GrayImage::new(img_dimensions.0*3, img_dimensions.1*3);
    
    println!("pic res: {} x {}", img_dimensions.0, img_dimensions.1);

    let mut sample_bytes: [u8; 4];
    let mut sample_array = Vec::<[u8; 4]>::new();
    
    //TODO: depending on wav bit size, change how it converts between signed to unsigned numbers
    match wav.spec().bits_per_sample {
        16 => {
            for sample in wav.samples::<i16>() {
                let current_sample = sample.as_ref().unwrap();
                let current_sample: i32 = *current_sample as i32;
                let current_sample = current_sample - i16::MIN as i32; //unsigned 24 bit MIN
                let current_sample = current_sample as u32;
                sample_bytes = current_sample.to_ne_bytes();
                sample_array.push(sample_bytes);

                assert_eq!(sample_bytes[2], 0);
            }
        }
        24 => {
                for sample in wav.samples::<i32>() {
                    let current_sample = sample.as_ref().unwrap();
                    let current_sample: i64 = *current_sample as i64;
                    let current_sample = current_sample - I24_MIN; //absolute value of unsigned 24 bit MIN
                    let current_sample = current_sample as u32;
                    sample_bytes = current_sample.to_ne_bytes();
                    sample_array.push(sample_bytes);

                    assert_eq!(sample_bytes[3], 0);
                }
        }
        _ => { println!("Unsupported bits per sample!"); return;}
    }

    let mut sample_iterator = sample_array.iter().peekable();

    let mut index_gray: usize = 0;
    if args.grayscale {
        for (_x, _y, pixel) in img.enumerate_pixels_mut() {

            if sample_iterator.peek() == None {
                pixel.0[1] = 128; // creates 0x8000 so converting back to wav creates silence
                continue;
            }

            let value = sample_iterator.peek().unwrap()[index_gray];
            
            pixel.0[0] = value;
            pixel.0[1] = value;
            pixel.0[2] = value;
            index_gray += 1;
            if wav.spec().bits_per_sample == 16 && index_gray >= 2 {index_gray = 0; sample_iterator.next();};

            if index_gray >= 3 {index_gray = 0; sample_iterator.next();};
            // println!("{}", rgb_array[0]);
        }
        img.save(&args.output).expect("Cant save file!")
    } else {
        for (_x, _y, pixel) in img.enumerate_pixels_mut() {
            
            if sample_iterator.peek() == None {
                pixel.0[2] = 128; // creates 0x800000 so converting back to wav creates silence
                continue;
            }

            let mut value = **sample_iterator.peek().unwrap();

            if wav.spec().bits_per_sample == 16 {
                // https://en.wikipedia.org/wiki/List_of_monochrome_and_RGB_color_formats#16-bit_RGB_(also_known_as_RGB565)
                //64172 = {11111010} {10101100}
                //         rrrrrggg   gggbbbbb      
                value.swap(0, 1); //incorrect otherwise idk why but whatever
                let mut value_r = value[0] >> 3;
                let mut value_g: u8 = (value[0].rotate_right(3) >> 5) * 2_u8.pow(3) + value[1] >> 5;
                let mut value_b = value[1].rotate_right(5) >> 3;
                value_r <<= 3; value_g <<= 2; value_b <<= 3; // to get colors from 0 - 255

                value[0] = value_r; value[1] = value_g; value[2] = value_b;
            }
            
            pixel.0[0] = value[0];
            pixel.0[1] = value[1];
            pixel.0[2] = value[2];

            pixel.0.rotate_left(args.rotate);

            sample_iterator.next();
        }
        img.save(&args.output).expect("Cant save file!")
    }
} 

fn img_to_wav(img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, path: &String, args: &Args) {

    let img_size = img.dimensions();
    let img_size = img_size.0 * img_size.1;
    let img_x = img.dimensions().0;
    let img_y = img.dimensions().1;
    println!("{}", img_size);

    let wav = hound::WavSpec {
        channels: 1 + args.stereo as u16,
        sample_rate: args.samplerate,
        bits_per_sample: 24 - (8 * args.bit16 as u16),
        sample_format: hound::SampleFormat::Int
    };

    println!("wav channels: {}", wav.channels);

    let mut wav = hound::WavWriter::create(&path, wav).expect("Cant save file");
    let mut pixel_array = Vec::<[u8; 4]>::new();

    if args.grayscale {
        let mut index_gray = 0;
        let mut pixel_gray: [u8; 3] = [0,0,0];
        if args.bit16 {
            for (_x, _y, pixel) in img.enumerate_pixels() {
                pixel_gray[index_gray] = pixel.0[0];
                index_gray += 1;
                if index_gray >= 2 {
                    pixel_array.push([pixel_gray[0], pixel_gray[1], 0, 0]);
                    index_gray = 0;
                }
            }
        } else {
            for (_x, _y, pixel) in img.enumerate_pixels() {
                pixel_gray[index_gray] = pixel.0[0];
                index_gray += 1;
                if index_gray >= 3 {
                    pixel_array.push([pixel_gray[0], pixel_gray[1], pixel_gray[2], 0]);
                    index_gray = 0;
                }
            }
        }
    } else {
        for (_x, _y, pixel) in img.enumerate_pixels() {
            let rgb: [u8; 3] = pixel.0;
            pixel_array.push([rgb[0], rgb[1], rgb[2], 0]);
        }   
    }

    if args.bit16 {
        for pixel in &pixel_array {
            let (mut r, mut g, mut b) = (pixel[0], pixel[1], pixel[2]);
            r >>= 3; g >>= 2; b >>= 3;
            let mut sample_to_write: u16 = r as u16;
            sample_to_write <<= 6;
            sample_to_write += g as u16;
            sample_to_write <<= 5;
            sample_to_write += b as u16;
            
            let sample_to_write: i32 = sample_to_write as i32 + i16::MIN as i32; //turn it into a value between 24bit integer MIN and MAX 
            wav.write_sample(sample_to_write).expect("Cant write sample");
        }
    } else {
        for pixel in &pixel_array {
            let sample_to_write = u32::from_ne_bytes(*pixel); //returns a 24 bit number
            let sample_to_write: i64 = sample_to_write as i64 + I24_MIN; //turn it into a value between 24bit integer MIN and MAX 
            let sample_to_write: i32 = sample_to_write as i32; //back again
            wav.write_sample(sample_to_write).expect("Cant write sample");
        }
    }

    wav.finalize().expect("cant save wav");

    let mut wav = File::open(&path).unwrap();
    let mut wav_buf = Vec::<u8>::new();
    wav.read_to_end(&mut wav_buf).unwrap();

    //writes a custom header where it specifes picture resolution 
    //looks bad maybe make it better in the future?
    let mut cursor = 0x04;
    let mut riff_size: [u8; 4] = [0, 0, 0, 0];
    riff_size[0] = *wav_buf.get(cursor).unwrap(); cursor += 1; 
    riff_size[1] = *wav_buf.get(cursor).unwrap(); cursor += 1; 
    riff_size[2] = *wav_buf.get(cursor).unwrap(); cursor += 1; 
    riff_size[3] = *wav_buf.get(cursor).unwrap();
    let mut riff_size: u32 = u32::from_ne_bytes(riff_size);
    riff_size += 16;

    let riff_size: [u8; 4] = riff_size.to_ne_bytes();
    let mut old_riff_size: [u8; 4] = [0, 0, 0, 0];

    cursor = 0x04;
    old_riff_size[0] = std::mem::replace(&mut wav_buf[cursor], riff_size[0]); cursor += 1;
    old_riff_size[1] = std::mem::replace(&mut wav_buf[cursor], riff_size[1]); cursor += 1;
    old_riff_size[2] = std::mem::replace(&mut wav_buf[cursor], riff_size[2]); cursor += 1;
    old_riff_size[3] = std::mem::replace(&mut wav_buf[cursor], riff_size[3]);

    println!("old riff size: {}, new riff size {}", u32::from_ne_bytes(old_riff_size), u32::from_ne_bytes(riff_size));

    let mut cursor = 0x3C;
    wav_buf.insert(cursor, b'i'); cursor += 1;
    wav_buf.insert(cursor, b'2'); cursor += 1;
    wav_buf.insert(cursor, b'w'); cursor += 1;
    wav_buf.insert(cursor, b' '); cursor += 1;
    wav_buf.insert(cursor, 8); cursor += 1;
    wav_buf.insert(cursor, 0); cursor += 1;
    wav_buf.insert(cursor, 0); cursor += 1;
    wav_buf.insert(cursor, 0); cursor += 1;
    wav_buf.insert(cursor, img_x.to_ne_bytes()[0]); cursor += 1;
    wav_buf.insert(cursor, img_x.to_ne_bytes()[1]); cursor += 1;
    wav_buf.insert(cursor, img_x.to_ne_bytes()[2]); cursor += 1;
    wav_buf.insert(cursor, img_x.to_ne_bytes()[3]); cursor += 1;
    wav_buf.insert(cursor, img_y.to_ne_bytes()[0]); cursor += 1;
    wav_buf.insert(cursor, img_y.to_ne_bytes()[1]); cursor += 1;
    wav_buf.insert(cursor, img_y.to_ne_bytes()[2]); cursor += 1;
    wav_buf.insert(cursor, img_y.to_ne_bytes()[3]);

    let mut wav = File::create(path).expect("cant create file");
    wav.write_all(&wav_buf).expect("cant write");
    
} 

struct Args {
    input: String,
    output: String,

    //img & wav args
    grayscale: bool,
    
    //wav2img args
    stereo: bool,
    
    //img2wav args
    dimensions: [i32; 2],
    rotate: usize,
    bit16: bool,
    samplerate: u32,

    is_wav: bool,
    is_img: bool
}
fn main() {

    let mut args_struct = Args{
        input: "".to_string(), 
        output: "".to_string(), 
        grayscale: false,
        stereo: false,
        dimensions: [0, 0],
        rotate: 0,
        bit16: false,
        samplerate: 44100,
        is_wav: false, 
        is_img: false,
    };

    if !process_args(&mut args_struct) {return;};

    if args_struct.input.is_empty() {println!("No input file provided"); return};
    if args_struct.output.is_empty() {println!("No output file provided"); return};

    //might be an unnecessary check
    if args_struct.input == args_struct.output {println!("Input and output can't be the same file!"); return};

    if args_struct.is_img {
        let img = ImageReader::open(&args_struct.input).expect("Cant open file").decode().unwrap().to_rgb8();
        img_to_wav(img, &args_struct.output, &args_struct);
    } else if args_struct.is_wav {
        wav_to_img(&args_struct.input.to_string(), &args_struct);
    } else {
        println!("File is not an image nor a wave file?");
        return
    }
}
