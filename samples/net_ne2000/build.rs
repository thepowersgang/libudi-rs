fn main() {
	let outpath = ::std::path::PathBuf::from( ::std::env::var("OUT_DIR").unwrap() ).join("udiprops.rs");

	let mut meta_bindings = ::std::collections::HashMap::new();

	let fp = ::std::io::BufReader::new(::std::fs::File::open("udiprops.txt").unwrap());
	for line in ::std::io::BufRead::lines(fp)
	{
		let line = line.unwrap();
		let line = line.trim();
		if line.len() == 0 || line.starts_with("#") {
			continue
		}
		let mut ents = line.split(' ');
		let e1 = ents.next().unwrap();
		if e1 == "meta" {
			let idx: u32 = ents.next().expect("`meta` needs two arguments, first missing").parse().expect("`meta` first arg must be int");
			let name = ents.next().expect("`meta` needs two arguments, second missing");
			meta_bindings.insert(idx, name.to_owned());
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
}


