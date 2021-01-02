# Import Pics

A small command line utility to import pictures and videos from a camera and arrange neatly in a separate folder for each date. The main motivations for writing it was to learn rust via a simple but somewhat useful project. It works well enough for me to use it, but be aware of its limitations if you decide to use it.

### Usage

The program was developed and compiled with Rust 1.48, but later versions should be fine, as well. Just clone the repo and run `cargo build` to get the executable, or `cargo build --release` to get an optimized executable. Type `import_pics --help` to get a description of the command line interface.

### Limitations

 - The program does not look at the content or metadata of existing files at the target location. If a file already exists where a file is supposed to be copied, the copy is skipped, and the file is never overwritten.
 - The utility uses the date created attribute of the file metadata instead of using exif attributes. This is in order to keep the utility simple and not to introduce any filetype restrictions. In practice it means that the dates refer to the end of a video/exposure instead of the beginning, and if files are copied or moved around before importing, the dates might be off. In most cases it should not be a big issue for the intended usage.
 - It assumes that the camera uses the correct time zone, and the folders are based on the time zone of the user's operating system. For example, if the user takes a Photo in London at 2020-12-31 22:00 and imports the pictures on a computer using London time it will be placed in a folder called 2021-01-01. It is intended behavior, but it would be nice to make it controllable with options.
  - Currently the folders are based on days. In the future an option may be added to create folders by weeks,months, years, etc.