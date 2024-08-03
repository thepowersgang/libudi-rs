//!
//! 
//! 
#![feature(str_split_whitespace_remainder)]

pub use self::parsed::Entry;
pub mod parsed;


 
/// Load from `udiprops.txt`
pub fn load_from_file(path: &::std::path::Path) -> ::std::io::Result< Vec<String> >
{
	let fp = ::std::io::BufReader::new(::std::fs::File::open(path)?);
    from_reader(fp)
}
/// Load from a file-like object
pub fn from_reader(fp: impl ::std::io::BufRead) -> ::std::io::Result< Vec<String> >
{
    let mut rv = Vec::new();

    let mut out_line = String::new();
	for line in ::std::io::BufRead::lines(fp)
	{
		let line = line?;
        if let Some(l) = get_line(&mut out_line, &line) {
            rv.push(l);
        }
	}

    Ok( rv )
}

/// Parse an input line and return the trimmed contents
/// 
/// The input has had all comments removed, and all useless whitespace replaced with a single space.
fn get_line(out_line: &mut String, line: &str) -> Option<String>
{
    // The “hash” character ('#') preceeds comments. Any '#', and any subsequent characters up to
    // the next line terminator are considered comments and will be completely ignored.
    let line = line.split('#').next().unwrap().trim();
    if line.len() == 0 {
        return if !out_line.is_empty() {
            Some(::core::mem::replace(out_line, String::new()))
        }
        else {
            None
        };
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
        None
    }
    else {
        Some(::core::mem::replace(out_line, String::new()))
    }
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

pub fn encode_to_raw(props: &[String]) -> Vec<u8> {
    props.iter()
        .flat_map(|v| v.as_bytes().iter().copied().chain(::std::iter::once(0)))
        .chain(::std::iter::once(0))
        .collect()
}

pub fn create_module_body(outfp: &mut dyn ::std::io::Write, props: &[String], emit_linkage: bool) -> ::std::io::Result<()>
{
    use ::std::collections::HashMap;
    use ::udi_sys::udi_index_t;
    struct State<'a,'p> {
        outfp: &'a mut dyn ::std::io::Write,

        meta_bindings: HashMap<&'p udi_index_t,&'p str>,
        regions: HashMap<&'p udi_index_t,()>,
        messages: HashMap<&'p parsed::MsgNum,(&'p str, Vec<&'static str>)>,
    }
    impl<'a,'p> State<'a,'p> {
        pub fn check_metalang(&mut self, linename: &'static str, meta_idx: &udi_index_t) -> ::std::io::Result<()> {
            if let None = self.meta_bindings.get(meta_idx) {
                writeln!(self.outfp, r#"compile_error!("`{linename}` references undefined metalang {:?}");"#, meta_idx)?;
            }
            Ok( () )
        }
        pub fn check_message(&mut self, linename: &'static str, message: &parsed::MsgNum) -> ::std::io::Result<()> {
            if let None = self.messages.get(message) {
                writeln!(self.outfp, r#"compile_error!("`{linename}` references undefined message {:?}");"#, message)?;
            }
            Ok( () )
        }
    }

    let parsed: Vec<_> = props.iter()
        .map(|line| match parsed::Entry::parse_line(line)
        {
        Ok(v) => v,
        Err(e) => {
            panic!("Malformed udiprops line {:?} - {:?}", line, e);
            },
        })
        .collect()
        ;
    let mut state = State {
        outfp,
        meta_bindings: Default::default(),
        regions: Default::default(),
        messages: Default::default(),
    };
	for ent in parsed.iter()
	{
        match ent
        {
        parsed::Entry::Metalang { meta_idx, interface_name } => {
			state.meta_bindings.insert(meta_idx, interface_name.to_owned());
            },
        parsed::Entry::Message(msg_id, body) => {

            // Parse the message for formatting fragments, and build up an argument set for type checking
            let mut p = ::udi_macro_helpers::printf::Parser::new(body.as_bytes());
            let mut types = Vec::new();
            loop {
                let e = match p.next() {
                    Ok(Some(e)) => e,
                    Ok(None) => break,
                    Err(e) => {
                        // TODO: Should this be a hard error?
                        writeln!(state.outfp, "compile_error!(\"Malformed message format string: {}\");", e.kind)?;
                        break;
                    },
                    };
                types.push(match e {
                udi_macro_helpers::printf::FormatArg::StringData(_) => continue,
                udi_macro_helpers::printf::FormatArg::Pointer(_) => "*const ::udi::ffi::c_void",
                udi_macro_helpers::printf::FormatArg::String(_, _) => "&::core::ffi::CStr",
                udi_macro_helpers::printf::FormatArg::BusAddr(_) => "::udi::ffi::physio::udi_busaddr64_t",
                udi_macro_helpers::printf::FormatArg::Char => "::udi::ffi::c_char",
                udi_macro_helpers::printf::FormatArg::Integer(_, _, ty, _) => match ty
                    {
                    udi_macro_helpers::printf::Size::U32 => "::udi::ffi::udi_ubit32_t",
                    udi_macro_helpers::printf::Size::U16 => "::udi::ffi::udi_ubit16_t",
                    udi_macro_helpers::printf::Size::U8 => "::udi::ffi::udi_ubit8_t",
                    },
                udi_macro_helpers::printf::FormatArg::BitSet(_) => "::udi::ffi::udi_ubit32_t",
                });
            }

            if let Some(_) = state.messages.insert(msg_id, (body,types)) {
                writeln!(state.outfp, "compile_error!(\"Duplicated message ID {:?}\");", msg_id)?;
            }
        },
        parsed::Entry::Region { region_idx, attributes } => {
            let _ = attributes;
            state.regions.insert(region_idx, ());
            },
        _ => {},
        }
    }

	for ent in parsed.iter()
	{
        match ent {
        // - Handled in first pass
        parsed::Entry::Metalang { .. } => {},
        parsed::Entry::Region { .. } => {},
        parsed::Entry::Message { .. } => {},

        // Check messages
        parsed::Entry::Supplier(message) => state.check_message("supplier", message)?,
        parsed::Entry::Contact(message) => state.check_message("contact", message)?,
        parsed::Entry::Name(message) => state.check_message("name", message)?,

        parsed::Entry::Device { device_name, meta_idx, attributes } => {
            state.check_message("device", device_name)?;
            state.check_metalang("device", meta_idx)?;
            for _ in attributes.clone() {
            }
        }

        parsed::Entry::ParentBindOps { meta_idx, region_idx, ops_idx, bind_cb_idx } => {
            // - Make sure that the metalang is present
            state.check_metalang("parent_bind_ops", meta_idx)?;
            // - Ensure that the region is defined
            if let None = state.regions.get(region_idx) {
                writeln!(state.outfp, r#"compile_error!("parent_bind_ops references undefined region {}");"#, region_idx)?;
            }
            // - Emit code that references the `define_driver` structs to make sure that `ops_idx`` binds with `bind_cb_idx`
            writeln!(state.outfp, r#"
fn _check_parent_bind_ops() {{
    let _ = <
        <super::OpsList::_{ops_idx} as ::udi::ops_markers::Ops>::OpsTy
        as
        ::udi::ops_markers::ParentBind< <super::CbList::_{bind_cb_idx} as ::udi::cb::CbDefinition >::Cb >
    >::ASSERT;
}}
"#)?;
            },
        parsed::Entry::ChildBindOps { meta_idx, region_idx, ops_idx } => {
            // - Make sure that the metalang is present
            state.check_metalang("child_bind_ops", meta_idx)?;
            // - Ensure that the region is defined
            if let None = state.regions.get(region_idx) {
                writeln!(state.outfp, r#"compile_error!("child_bind_ops references undefined region {}");"#, region_idx)?;
            }
            // - Emit code that references the `define_driver` structs to make sure that `ops_idx`` binds with `bind_cb_idx`
            writeln!(state.outfp, r#"
fn _check_child_bind_ops() {{
    let _ = <
        <super::OpsList::_{ops_idx} as ::udi::ops_markers::Ops>::OpsTy
        as
        ::udi::ops_markers::ChildBind
    >::ASSERT;
}}
"#)?;
            },
        _ => {},
		}
	}

    //writeln!(state.outfp, "mod messages {{")?;
    for (msgnum,(_body,types)) in state.messages {
        writeln!(state.outfp, "pub struct Msg{};", msgnum.0)?;
        writeln!(state.outfp, "impl ::udi::log::Message for Msg{} {{", msgnum.0)?;
        writeln!(state.outfp, "  const NUM: ::udi::ffi::udi_ubit32_t = {};", msgnum.0)?;
        write!(state.outfp, "  type Args = (")?;
        for t in types {
            write!(state.outfp, "{},", t)?;
        }
        writeln!(state.outfp, ");")?;
        writeln!(state.outfp, "}}")?;
    }
    //writeln!(state.outfp, "}}")?;

    let outfp = state.outfp;
	writeln!(outfp, "#[allow(non_upper_case_globals)]")?;
	writeln!(outfp, "pub mod meta {{")?;
	for (idx,name) in state.meta_bindings {
		writeln!(outfp, "pub const {}: ::udi::ffi::udi_index_t = ::udi::ffi::udi_index_t({});", name, idx.0)?;
	}
	writeln!(outfp, "}}")?;

    // --- Emit `udiprops.txt` as a valid `.udiprops` section (NUL terminated strings)
    let udiprops_encoded: Vec<u8> = encode_to_raw(props);
    if emit_linkage {
        writeln!(outfp, "#[allow(non_upper_case_globals,dead_code)]")?;
        writeln!(outfp, "#[link_section=\".udiprops\"]")?;
        writeln!(outfp, "#[export_name=\"libudi_rs_udiprops\"]")?;
    }
    else {
        writeln!(outfp, "#[allow(non_upper_case_globals)]")?;
    }
    writeln!(outfp, "pub static udiprops: [u8; {}] = *b\"{}\";", udiprops_encoded.len(), ByteStrDump(&udiprops_encoded))?;
    // HACK for testing
    if emit_linkage {
        writeln!(outfp, "#[export_name=\"libudi_rs_udiprops_len\"]")?;
        writeln!(outfp, "pub static _LEN: usize = {};", udiprops_encoded.len())?;
    }


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

    Ok( () )
}

/// Load `udiprops.txt` and emit `$OUT_DIR/udiprops.rs`
pub fn build_script()
{
	let outpath = ::std::path::PathBuf::from( ::std::env::var("OUT_DIR").unwrap() ).join("udiprops.rs");

    let props = load_from_file("udiprops.txt".as_ref()).expect("Unable to load `udiprops.txt`");

	let mut outfp = ::std::io::BufWriter::new( ::std::fs::File::create( outpath).unwrap() );

    create_module_body(&mut outfp, &props, true).unwrap();
}