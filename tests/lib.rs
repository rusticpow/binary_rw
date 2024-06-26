use anyhow::Result;
use binary_rw::{
    BinaryReader, BinaryWriter, Endian, FileStream, MemoryStream, SeekStream, SliceStream,
};

fn create_writer_stream(name: &str) -> FileStream {
    let name = format!("{}.test", name);
    FileStream::create(&name).expect("Failed to open stream")
}

fn create_reader_stream(name: &str) -> FileStream {
    let name = format!("{}.test", name);
    FileStream::open(&name).expect("Failed to open stream")
}

fn cleanup(name: &str) {
    let name = format!("{}.test", name);
    std::fs::remove_file(&name).expect("Failure to delete file");
}

#[test]
fn borrow_test() -> Result<()> {
    let mut stream = MemoryStream::new();
    let mut writer = BinaryWriter::new(&mut stream, Endian::Big);
    writer.write_u8(8)?;
    writer.write_u8(&8)?;
    writer.write_i8(-8)?;
    writer.write_i8(&-8)?;

    writer.write_u16(16)?;
    writer.write_u16(&16)?;
    writer.write_i16(-16)?;
    writer.write_i16(&-16)?;

    writer.write_u32(32)?;
    writer.write_u32(&32)?;
    writer.write_i32(-32)?;
    writer.write_i32(&-32)?;

    writer.write_u64(64)?;
    writer.write_u64(&64)?;
    writer.write_i64(-64)?;
    writer.write_i64(&-64)?;

    writer.write_u128(128)?;
    writer.write_u128(&128)?;
    writer.write_i128(-128)?;
    writer.write_i128(&-128)?;

    writer.write_usize(64)?;
    writer.write_usize(&64)?;
    writer.write_isize(-64)?;
    writer.write_isize(&-64)?;

    writer.write_char('c')?;
    writer.write_char(&'c')?;

    writer.write_bool(true)?;
    writer.write_bool(&true)?;

    writer.write_string("foo")?;
    writer.write_string(String::from("foo"))?;

    let buf: Vec<u8> = vec![1, 2, 3, 4];
    let exp: Vec<u8> = buf.clone(); // for assertion

    writer.write_bytes(&buf)?;
    writer.write_bytes(buf)?;

    let buffer: Vec<u8> = stream.into();

    let mut stream = SliceStream::new(&buffer);
    let mut reader = BinaryReader::new(&mut stream, Endian::Big);

    let value = (reader.read_u8()?, reader.read_u8()?);
    assert_eq!((8, 8), value);
    let value = (reader.read_i8()?, reader.read_i8()?);
    assert_eq!((-8, -8), value);

    let value = (reader.read_u16()?, reader.read_u16()?);
    assert_eq!((16, 16), value);
    let value = (reader.read_i16()?, reader.read_i16()?);
    assert_eq!((-16, -16), value);

    let value = (reader.read_u32()?, reader.read_u32()?);
    assert_eq!((32, 32), value);
    let value = (reader.read_i32()?, reader.read_i32()?);
    assert_eq!((-32, -32), value);

    let value = (reader.read_u64()?, reader.read_u64()?);
    assert_eq!((64, 64), value);
    let value = (reader.read_i64()?, reader.read_i64()?);
    assert_eq!((-64, -64), value);

    let value = (reader.read_u128()?, reader.read_u128()?);
    assert_eq!((128, 128), value);
    let value = (reader.read_i128()?, reader.read_i128()?);
    assert_eq!((-128, -128), value);

    let value = (reader.read_usize()?, reader.read_usize()?);
    assert_eq!((64, 64), value);
    let value = (reader.read_isize()?, reader.read_isize()?);
    assert_eq!((-64, -64), value);

    let value = (reader.read_char()?, reader.read_char()?);
    assert_eq!(('c', 'c'), value);

    let value = (reader.read_bool()?, reader.read_bool()?);
    assert_eq!((true, true), value);

    let value = (reader.read_string()?, reader.read_string()?);
    assert_eq!((String::from("foo"), String::from("foo")), value);

    let value = (reader.read_bytes(4)?, reader.read_bytes(4)?);
    assert_eq!((exp.clone(), exp), value);

    Ok(())
}

#[test]
fn slice_test() -> Result<()> {
    let size: usize = if cfg!(any(feature = "wasm32", feature = "string_len_u32")) {
        19
    } else {
        23
    };

    let mut stream = MemoryStream::new();
    let mut writer = BinaryWriter::new(&mut stream, Endian::Big);
    writer.write_u32(42)?;
    writer.write_string("foo")?;
    writer.write_7bit_encoded_len_string("bar")?;
    writer.write_char('b')?;

    assert_eq!(size, writer.len()?);

    let buffer: Vec<u8> = stream.into();

    let mut stream = SliceStream::new(&buffer);
    let mut reader = BinaryReader::new(&mut stream, Endian::Big);

    reader.seek(0)?;
    let value = reader.read_u32()?;
    assert_eq!(42, value);

    assert_eq!(4, reader.tell()?);

    let value = reader.read_string()?;
    assert_eq!("foo", &value);

    let value = reader.read_7bit_encoded_len_string()?;
    assert_eq!("bar", &value);

    let value = reader.read_char()?;
    assert_eq!('b', value);

    assert_eq!(size, reader.len()?);

    Ok(())
}

#[test]
fn seek_test() -> Result<()> {
    let temp: f32 = 50.0;
    let seek_loc = 5;

    let mut stream = create_writer_stream("seek");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_bytes([16; 32].to_vec())?;
    writer.seek(seek_loc)?;
    assert_eq!(writer.tell()?, seek_loc);
    writer.write_f32(temp)?;

    let mut stream = create_reader_stream("seek");
    let mut reader = BinaryReader::new(&mut stream, Default::default());
    reader.seek(seek_loc)?;
    assert_eq!(reader.tell()?, seek_loc);
    let read_temp = reader.read_f32()?;

    assert_eq!(temp, read_temp);

    cleanup("seek");

    Ok(())
}

#[test]
fn read_write_test_f64() -> Result<()> {
    let temp: f64 = 50.0;
    let mut stream = create_writer_stream("f64");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_f64(temp)?;

    let mut stream = create_reader_stream("f64");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_f64()?;

    assert_eq!(temp, read_temp);

    cleanup("f64");
    Ok(())
}

#[test]
fn read_write_test_f32() -> Result<()> {
    let temp: f32 = 50.0;
    let mut stream = create_writer_stream("f32");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_f32(temp)?;

    let mut stream = create_reader_stream("f32");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_f32()?;

    assert_eq!(temp, read_temp);

    cleanup("f32");

    Ok(())
}

#[test]
fn read_write_test_isize() -> Result<()> {
    let temp: isize = 50;
    let mut stream = create_writer_stream("isize");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_isize(temp)?;

    let mut stream = create_reader_stream("isize");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_isize()?;

    assert_eq!(temp, read_temp);

    cleanup("isize");

    Ok(())
}

#[test]
fn read_write_test_usize() -> Result<()> {
    let temp: usize = 50;
    let mut stream = create_writer_stream("usize");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_usize(temp)?;

    let mut stream = create_reader_stream("usize");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_usize()?;
    assert_eq!(temp, read_temp);

    cleanup("usize");

    Ok(())
}

#[test]
fn read_write_test_i128() -> Result<()> {
    let temp: i128 = 1 << 127;
    let mut stream = create_writer_stream("i128");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_i128(temp)?;

    let mut stream = create_reader_stream("i128");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_i128()?;

    assert_eq!(temp, read_temp);

    cleanup("i128");

    Ok(())
}

#[test]
fn read_write_test_u128() -> Result<()> {
    let temp: u128 = 1 << 127;
    let mut stream = create_writer_stream("u128");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_u128(temp)?;

    let mut stream = create_reader_stream("u128");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_u128()?;

    assert_eq!(temp, read_temp);

    cleanup("u128");

    Ok(())
}

#[test]
fn read_write_test_i64() -> Result<()> {
    let temp: i64 = 50;
    let mut stream = create_writer_stream("i64");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_i64(temp)?;

    let mut stream = create_reader_stream("i64");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_i64()?;

    assert_eq!(temp, read_temp);

    cleanup("i64");

    Ok(())
}

#[test]
fn read_write_test_i32() -> Result<()> {
    let temp: i32 = 50;
    let mut stream = create_writer_stream("i32");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_i32(temp)?;

    let mut stream = create_reader_stream("i32");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_i32()?;

    assert_eq!(temp, read_temp);

    cleanup("i32");
    Ok(())
}

#[test]
fn read_write_test_i16() -> Result<()> {
    let temp: i16 = 50;
    let mut stream = create_writer_stream("i16");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_i16(temp)?;

    let mut stream = create_reader_stream("i16");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_i16()?;

    assert_eq!(temp, read_temp);

    cleanup("i16");

    Ok(())
}

#[test]
fn read_write_test_i8() -> Result<()> {
    let temp: i8 = 50;
    let mut stream = create_writer_stream("i8");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_i8(temp)?;

    let mut stream = create_reader_stream("i8");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_i8()?;

    assert_eq!(temp, read_temp);

    cleanup("i8");

    Ok(())
}

#[test]
fn read_write_test_u64() -> Result<()> {
    let temp: u64 = 50;
    let mut stream = create_writer_stream("u64");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_u64(temp)?;

    let mut stream = create_reader_stream("u64");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_u64()?;

    assert_eq!(temp, read_temp);

    cleanup("u64");

    Ok(())
}

#[test]
fn read_write_test_u32() -> Result<()> {
    let temp: u32 = 50;
    let mut stream = create_writer_stream("u32");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_u32(temp)?;

    let mut stream = create_reader_stream("u32");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_u32()?;

    assert_eq!(temp, read_temp);

    cleanup("u32");

    Ok(())
}

#[test]
fn read_write_test_u16() -> Result<()> {
    let temp: u16 = 50;
    let mut stream = create_writer_stream("u16");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_u16(temp)?;

    let mut stream = create_reader_stream("u16");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_u16()?;

    assert_eq!(temp, read_temp);

    cleanup("u16");

    Ok(())
}

#[test]
fn read_write_test_u8() -> Result<()> {
    let temp: u8 = 50;
    let mut stream = create_writer_stream("u8");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_u8(temp)?;

    let mut stream = create_reader_stream("u8");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_temp = reader.read_u8()?;

    assert_eq!(temp, read_temp);

    cleanup("u8");

    Ok(())
}

#[test]
fn read_write_test_7bit_encoded_u128() -> Result<()> {
    let values: [(u128, usize); 6] = [(50, 1), (270, 2), (70000, 3), (2147483647, 5), (1 << 63, 10), (1 << 127, 19)];

    for (temp, size_expected) in values {
        let mut stream = create_writer_stream("7bit_encoded_u128");
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let size = writer.write_7bit_encoded_u128(temp)?;
        assert_eq!(size_expected, size);

        let mut stream = create_reader_stream("7bit_encoded_u128");
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let read_temp = reader.read_7bit_encoded_u128()?;

        assert_eq!(temp, read_temp);

        cleanup("7bit_encoded_u128");
    }
    Ok(())
}


#[test]
fn read_write_test_7bit_encoded_i128() -> Result<()> {
    let values: [(i128, usize); 8] = [(-2147483647, 19), (-100, 19), (50, 1), (270, 2), (70000, 3), (2147483647, 5), (1 << 63, 10), (1 << 127, 19)];
    for (temp, size_expected) in values {

        let mut stream = create_writer_stream("7bit_encoded_i128");
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let size = writer.write_7bit_encoded_i128(temp)?;
        assert_eq!(size_expected, size);

        let mut stream = create_reader_stream("7bit_encoded_i128");
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let read_temp = reader.read_7bit_encoded_i128()?;

        assert_eq!(temp, read_temp);

        cleanup("7bit_encoded_i128");
    }
    Ok(())
}

#[test]
fn read_write_test_7bit_encoded_u64() -> Result<()> {
    let values: [(u64, usize); 5] = [(50, 1), (270, 2), (70000, 3), (2147483647, 5), (1 << 63, 10)];

    for (temp, size_expected) in values {
        let mut stream = create_writer_stream("7bit_encoded_u64");
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let size = writer.write_7bit_encoded_u64(temp)?;
        assert_eq!(size_expected, size);

        let mut stream = create_reader_stream("7bit_encoded_u64");
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let read_temp = reader.read_7bit_encoded_u64()?;

        assert_eq!(temp, read_temp);

        cleanup("7bit_encoded_u64");
    }
    Ok(())
}

#[test]
fn read_write_test_7bit_encoded_i64() -> Result<()> {
    let values: [(i64, usize); 7] = [(-2147483647, 10), (-100, 10), (50, 1), (270, 2), (70000, 3), (2147483647, 5), (1 << 63, 10)];

    for (temp, size_expected) in values {
        let mut stream = create_writer_stream("7bit_encoded_i64");
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let size = writer.write_7bit_encoded_i64(temp)?;
        assert_eq!(size_expected, size);

        let mut stream = create_reader_stream("7bit_encoded_i64");
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let read_temp = reader.read_7bit_encoded_i64()?;

        assert_eq!(temp, read_temp);

        cleanup("7bit_encoded_i64");
    }
    Ok(())
}

#[test]
fn read_write_test_7bit_encoded_i32() -> Result<()> {
    let values: [(i32, usize); 6] = [(-2147483647, 5), (-100, 5), (50, 1), (270, 2), (70000, 3), (2147483647, 5)];

    for (temp, size_expected) in values {
        let mut stream = create_writer_stream("7bit_encoded_i32");
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let size = writer.write_7bit_encoded_i32(temp)?;
        assert_eq!(size_expected, size);

        let mut stream = create_reader_stream("7bit_encoded_i32");
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let read_temp = reader.read_7bit_encoded_i32()?;

        assert_eq!(temp, read_temp);

        cleanup("7bit_encoded_i32");
    }
    Ok(())
}

#[test]
fn read_write_test_7bit_encoded_u32() -> Result<()> {
    let values: [(u32, usize); 4] = [(50, 1), (270, 2), (70000, 3), (2147483647, 5)];

    for (temp, size_expected) in values {
        let mut stream = create_writer_stream("7bit_encoded_u32");
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let size = writer.write_7bit_encoded_u32(temp)?;
        assert_eq!(size_expected, size);

        let mut stream = create_reader_stream("7bit_encoded_u32");
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let read_temp = reader.read_7bit_encoded_u32()?;

        assert_eq!(temp, read_temp);

        cleanup("7bit_encoded_u32");
    }
    Ok(())
}

#[test]
fn read_write_test_7bit_encoded_usize() -> Result<()> {
    let values: [(usize, usize); 4] = [(50, 1), (270, 2), (70000, 3), (2147483647, 5)];

    for (temp, size_expected) in values {
        let mut stream = create_writer_stream("7bit_encoded_usize");
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let size = writer.write_7bit_encoded_usize(temp)?;
        assert_eq!(size_expected, size);

        let mut stream = create_reader_stream("7bit_encoded_usize");
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let read_temp = reader.read_7bit_encoded_usize()?;

        assert_eq!(temp, read_temp);

        cleanup("7bit_encoded_usize");
    }
    Ok(())
}

#[test]
fn read_write_bytes() -> Result<()> {
    let count = 20;

    let temp = vec![16; count];

    let mut stream = create_writer_stream("bytes");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_bytes(temp.clone())?;

    let mut stream = create_reader_stream("bytes");
    let mut reader = BinaryReader::new(&mut stream, Default::default());
    let read_temp = reader.read_bytes(count)?;

    assert_eq!(temp, read_temp);

    cleanup("bytes");

    Ok(())
}

#[test]
fn read_out_of_range() -> Result<()> {
    let mut stream = create_writer_stream("out_of_range");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_f32(5.0)?;

    let mut stream = create_reader_stream("out_of_range");
    let mut reader = BinaryReader::new(&mut stream, Default::default());
    reader.read_f32()?;

    assert!(reader.read_f32().is_err());

    cleanup("out_of_range");
    Ok(())
}

#[test]
fn read_write_string() -> Result<()> {
    let temp = "Hello World";
    let mut stream = create_writer_stream("read_write_string");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_string(temp.to_string())?;

    let mut stream = create_reader_stream("read_write_string");
    let mut reader = BinaryReader::new(&mut stream, Default::default());
    let string = reader.read_string()?;
    assert_eq!(temp, string);

    cleanup("read_write_string");
    Ok(())
}

#[test]
fn read_write_7bit_encoded_string() -> Result<()> {
    let temp = "Hello World";
    let mut stream = create_writer_stream("read_7bit_encoded_len_string");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_7bit_encoded_len_string(temp.to_string())?;

    let mut stream = create_reader_stream("read_7bit_encoded_len_string");
    let mut reader = BinaryReader::new(&mut stream, Default::default());
    let string = reader.read_7bit_encoded_len_string()?;
    assert_eq!(temp, string);

    cleanup("read_7bit_encoded_len_string");
    Ok(())
}

#[test]
fn read_write_test_bool() -> Result<()> {
    let positive = true;
    let negative = false;
    let mut stream = create_writer_stream("bool");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());

    writer.write_bool(positive)?;
    writer.write_bool(negative)?;

    let mut stream = create_reader_stream("bool");
    let mut reader = BinaryReader::new(&mut stream, Default::default());

    let read_positive = reader.read_bool()?;
    let read_negative = reader.read_bool()?;

    assert_eq!(positive, read_positive);
    assert_eq!(negative, read_negative);

    cleanup("bool");
    Ok(())
}

#[test]
fn read_write_from_memorystream() -> Result<()> {
    let value_a = 3.0;
    let value_b = 5.0;
    let mut stream = MemoryStream::new();
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_f32(value_a)?;
    writer.write_f32(value_b)?;

    let mut reader = BinaryReader::new(&mut stream, Default::default());
    reader.seek(0)?;
    let value = reader.read_f32()?;
    assert_eq!(value_a, value);
    let value = reader.read_f32()?;
    assert_eq!(value_b, value);

    Ok(())
}

#[test]
fn write_to_memorystream_overlapping() -> Result<()> {
    let mut stream = MemoryStream::new();
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_f32(1.0)?;
    writer.write_f32(2.0)?;
    writer.write_f32(3.0)?;

    writer.seek(0)?;
    writer.write_f32(4.0)?;
    writer.write_f32(5.0)?;
    writer.write_f32(6.0)?;

    let mut reader = BinaryReader::new(&mut stream, Default::default());
    reader.seek(0)?;
    let value = reader.read_f32()?;
    assert_eq!(4.0, value);
    let value = reader.read_f32()?;
    assert_eq!(5.0, value);
    let value = reader.read_f32()?;
    assert_eq!(6.0, value);

    Ok(())
}

#[test]
fn write_to_memorystream_into_vec() -> Result<()> {
    let mut stream = MemoryStream::new();
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_f32(1.0)?;
    let vec: Vec<u8> = stream.into();
    assert_eq!(4, vec.len());
    Ok(())
}

#[test]
fn write_bytes_with_value() -> Result<()> {
    let mut stream = MemoryStream::new();
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_bytes_with_value(3, 1)?;

    assert_eq!(3, writer.len()?);
    Ok(())
}

#[test]
fn swap_endianness_swaps() -> Result<()> {
    let mut stream = MemoryStream::new();
    {
        let mut writer = BinaryWriter::new(&mut stream, Endian::Big);
        writer.write_i32(1)?;
    }
    stream.seek(0)?;
    let mut reader: BinaryReader = BinaryReader::new(&mut stream, Endian::Little);

    assert_ne!(1, reader.read_i32()?);

    reader.seek(0)?;

    reader.swap_endianness();

    assert_eq!(1, reader.read_i32()?);

    Ok(())
}

#[test]
fn write_to_filestream_overlapping() -> Result<()> {
    let mut stream = create_writer_stream("filestream_overlapping");
    let mut writer = BinaryWriter::new(&mut stream, Default::default());
    writer.write_f32(1.0)?;
    writer.write_f32(2.0)?;
    writer.write_f32(3.0)?;

    writer.seek(0)?;
    writer.write_f32(4.0)?;
    writer.write_f32(5.0)?;
    writer.write_f32(6.0)?;

    let file = std::fs::File::open("filestream_overlapping.test")?;
    eprintln!("File size is {}", file.metadata()?.len());

    let mut stream = create_reader_stream("filestream_overlapping");
    let mut reader = BinaryReader::new(&mut stream, Default::default());
    let value = reader.read_f32()?;
    assert_eq!(4.0, value);
    let value = reader.read_f32()?;
    assert_eq!(5.0, value);
    let value = reader.read_f32()?;
    assert_eq!(6.0, value);

    cleanup("filestream_overlapping");

    Ok(())
}
