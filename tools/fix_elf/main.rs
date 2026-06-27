//
// Rewrite an ELF file to convert DT_NEEDED to DT_SYMBOLIC
//
// cspell:ignore phent phents phdr filesize

const DT_NEEDED: u64 = 1;
const DT_SYMBOLIC: u8 = 16;
const PT_DYNAMIC: [u8; 4] = [2,0,0,0];

fn main() {
    let input_path = {
        let mut a = ::std::env::args_os();
        // Skip the executable name
        a.next();
        a.next().unwrap()
        };
    let mut file_contents = Vec::new();
    ::std::io::Read::read_to_end(&mut ::std::fs::File::open(&input_path).unwrap(), &mut file_contents).unwrap();

    // 1. Check type (magic bytes, including the variant (Elf32) and endian (little) flags)
    if !file_contents.starts_with(b"\x7FELF\x02\x01") { // cspell:disable-line
        panic!("Bad magic")
    }
    let phdr_off = u64::from_bytes( &file_contents[32..] );
    let n_phent   = u16::from_bytes( &file_contents[58..] );
    // 2. Find the dynamic section
    let Some((p_offset, p_filesize)) = find_pt_dynamic(&file_contents[phdr_off as usize..], n_phent) else {
        panic!("No PT_DYNAMIC");
    };
    // Rewrite contents to convert DT_NEEDED to DT_SYMBOLIC
    for chunk in file_contents[p_offset as usize..][..p_filesize as usize].chunks_mut(16) {
        let d_tag = u64::from_bytes(&chunk[0..]);
        //let d_value = u64::from_bytes(&chunk[8..]);
        if d_tag == DT_NEEDED {
            // Just update the first byte (rest are known to be 0)
            chunk[0] = DT_SYMBOLIC;
        }
        else if d_tag == 0 {
            break;
        }
    }

    ::std::io::Write::write_all(&mut ::std::fs::File::create(input_path).unwrap(), &file_contents).unwrap();
}

fn find_pt_dynamic(b_dynamic: &[u8], n_phent: u16) -> Option<(u64,u64)> {
    let phents = b_dynamic.chunks(56).take(n_phent as usize);
    for phent in phents {
        if phent[..4] == PT_DYNAMIC {
            let p_offset = u64::from_bytes(&phent[8..]);
            let p_filesize = u64::from_bytes(&phent[32..]);
            return Some((p_offset, p_filesize));
        }
    }
    None
}

trait GetPrimLe {
    fn from_bytes(b: &[u8]) -> Self;
}
impl GetPrimLe for u16 {
    fn from_bytes(b: &[u8]) -> Self {
        Self::from_le_bytes( b[..2].try_into().unwrap() )
    }
}
impl GetPrimLe for u32 {
    fn from_bytes(b: &[u8]) -> Self {
        Self::from_le_bytes( b[..4].try_into().unwrap() )
    }
}
impl GetPrimLe for u64 {
    fn from_bytes(b: &[u8]) -> Self {
        Self::from_le_bytes( b[..8].try_into().unwrap() )
    }
}