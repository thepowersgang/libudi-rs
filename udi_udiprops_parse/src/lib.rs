//!
//! 
//! 

pub use self::parsed::Entry;
pub mod parsed;


/// Load from `udiprops.txt`
pub fn load_from_file(path: &::std::path::Path) -> ::std::io::Result< Vec<String> >
{
    let mut rv = Vec::new();

	let fp = ::std::io::BufReader::new(::std::fs::File::open(path)?);
    let mut out_line = String::new();
	for line in ::std::io::BufRead::lines(fp)
	{
		let line = line?;
		//let line = line.trim();
		//if line.len() == 0 || line.starts_with("#") {
		//	continue
		//}

        // The “hash” character ('#') preceeds comments. Any '#', and any subsequent characters up to
        // the next line terminator are considered comments and will be completely ignored.
		let line = line.split('#').next().unwrap().trim();
		if line.len() == 0 {
			continue
		}

		let mut ents = line.split_whitespace();
		let e1 = ents.next().expect("Empty string after empty check?!");
        out_line.push_str(e1);
        for v in ents {
            if v.starts_with('#') {
                break;
            }
            if ! v.is_empty() {
                out_line.push(' ');
                out_line.push_str(v);
            }
        }
        // If the last non-comment character on a line is a backslash (‘\’) and is not immediately
        // preceded by another backslash character, then the backslash and the line terminator are
        // ignored, and this line and the following line are treated as a single logical line. Any
        // whitespace immediately preceding the backslash becomes part of the logical line and is not
        // ignored. The total length of a logical line, including all backslashes and line terminators,
        // must be less than 512 bytes long.
        if out_line.ends_with('\\') && !out_line.ends_with("\\\\") {
            continue ;
        }
        rv.push(out_line);
        out_line = String::new();
	}

    Ok( rv )
}

/// An iterator over a compacted encoding stored in `.udiprops`
#[derive(Clone)]
pub struct EncodedIter<'a>(&'a [u8]);
impl<'a> ::core::iter::Iterator for EncodedIter<'a>
{
    type Item = parsed::Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Ignore/consume any blank lines
        // - They're valid, but unlikely with the encoder in this library.
        // - Also, the `len..` below doesn't consume the NUL byte
        while let Some((&0,rest)) = self.0.split_first() {
            self.0 = rest;
        }

        // Once the slice is empty, we've reached the end
        if self.0.is_empty() {
            None
        }
        else {
            let len = self.0.iter().position(|v| *v == 0).unwrap_or(self.0.len());
            let bytes = &self.0[..len];
            self.0 = &self.0[len..];
            
            let str = ::core::str::from_utf8(bytes).expect("Invalid UTF-8 in udiprops section?!");
            Some( parsed::Entry::parse_line(str).expect("Malformed line in udiprops section") )
        }
    }
}
/// Load from a blob of memory (`.udiprops` section)
pub fn load_from_raw_section(data: &[u8]) -> EncodedIter {
    EncodedIter(data)
}

/// Load `udiprops.txt` and emit `$OUT_DIR/udiprops.rs`
pub fn build_script()
{
	let outpath = ::std::path::PathBuf::from( ::std::env::var("OUT_DIR").unwrap() ).join("udiprops.rs");
	let mut meta_bindings = ::std::collections::HashMap::new();

    let props = load_from_file("udiprops.txt".as_ref()).expect("Unable to load `udiprops.txt`");

	for line in props.iter()
	{
        let ent = match parsed::Entry::parse_line(line)
            {
            Ok(v) => v,
            Err(e) => {
                panic!("Malformed udiprops line {:?} - {:?}", line, e);
                },
            };
        match ent
        {
        parsed::Entry::Metalang { meta_idx, interface_name } => {
			meta_bindings.insert(meta_idx, interface_name.to_owned());
            },
        _ => {},
		}
	}

	use ::std::io::Write;
	let mut outfp = ::std::io::BufWriter::new( ::std::fs::File::create( outpath).unwrap() );

	writeln!(outfp, "#[allow(non_upper_case_globals)]").unwrap();
	writeln!(outfp, "pub mod meta {{").unwrap();
	for (idx,name) in meta_bindings {
		writeln!(outfp, "pub const {}: ::udi::ffi::udi_index_t = {};", name, idx).unwrap();
	}
	writeln!(outfp, "}}").unwrap();

    // --- Emit `udiprops.txt` as a valid `.udiprops` section (NUL terminated strings)
    let udiprops_encoded: Vec<u8> = props.iter()
    .flat_map(|v| v.as_bytes().iter().copied().chain(::std::iter::once(0)))
    .chain(::std::iter::once(0))
    .collect();
	writeln!(outfp, "#[allow(non_upper_case_globals,dead_code)]").unwrap();
    writeln!(outfp, "#[link_section=\".udiprops\"]").unwrap();
    writeln!(outfp, "#[export_name=\"libudi_rs_udiprops\"]").unwrap();
    writeln!(outfp, "pub static udiprops: [u8; {}] = *b\"{}\";", udiprops_encoded.len(), ByteStrDump(&udiprops_encoded)).unwrap();
    // HACK for testing
    writeln!(outfp, "#[export_name=\"libudi_rs_udiprops_len\"]").unwrap();
    writeln!(outfp, "pub static _LEN: usize = {};", udiprops_encoded.len()).unwrap();


    struct ByteStrDump<'a>(&'a [u8]);
    impl ::core::fmt::Display for ByteStrDump<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for &b in self.0 {
                match b {
                0 => f.write_str("\\0")?,
                b'"' => f.write_str("\\\"")?,
                0x20 ..= 0x7E => write!(f, "{}", b as char)?,
                _ => write!(f, "\\x{:02x}", b)?,
                }
            }
            Ok( () )
        }
    }
}