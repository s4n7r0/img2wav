# img2wav
Converts audio samples to RGB pixels and vice-versa

![](https://github.com/s4n7r0/img2wav/blob/main/preview.gif)

Supported formats: 16-bit .wav, 24-bit .wav, .png, .jpg, .jpeg, .gif, .bmp, .webp

Converting a 16bit wav produces an image in RGB565 mode

### Dependencies
[Hound](https://crates.io/crates/hound)

[image](https://crates.io/crates/image)

### Building
```
git clone https://github.com/s4n7r0/img2wav/
cd ./img2wav
cargo build
```

## Usage
```
Usage: filename [INPUT] [OPTIONS]
or drag'n'drop a supported file on filename

Options:
      -i, --input  [PATH]                   Input file
      -o, --output [PATH]                   Output file
      -g, --grayscale                       Process input in grayscale (Byte by byte)
If converting from wav to image:
      -d, --dimensions [WIDTH]x[HEIGHT]     Dimensions of output image
      -g, --grayscale                       Outputs image in grayscale
      -r, --rotate [VALUE]                  Rotates RGB
If converting from image to wav
      -s, --stereo                          Outputs audio with 2 channels
      -g, --grayscale                       Interpretes image as if it was in Grayscale
      -16                                   Outputs audio in 16bit, assuming that image is in RGB565 mode
```
