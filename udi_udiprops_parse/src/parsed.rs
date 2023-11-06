use ::udi_sys::udi_index_t;

/// A parsed entry in a udiprops
#[derive(Debug)]
pub enum Entry<'a> {
    PropertiesVersion(Version),
    Supplier(MsgNum),
    Contact(MsgNum),
    Name(MsgNum),

    Shortname(&'a str),
    Release { sequence_number: u32, release_string: &'a str },
    /// Require an interface of a specified compatible version
    Requires(&'a str, Version),
    Module { filename: &'a str },
    Locale { locale: &'a str },
    Message(MsgNum, &'a str),
    /// A message to be used in emergency cases (allows an envionment to lazily load non-critical messages)
    DiasterMessage(MsgNum, &'a str),
    /// An external udiprops.txt-style file containing messages
    MessageFile(&'a str),

    Provides {
        interface_name: &'a str,
        version_number: Version,
        // C header file list
    },
    Symbols {
        library_symbol: Option<&'a str>,
        provided_symbol: &'a str,
    },
    Category(MsgNum),

    // ----
    Metalang {
        meta_idx: udi_index_t,
        interface_name: &'a str,    // Must match a `requires`
    },
    ChildBindOps {
        meta_idx: udi_index_t,
        region_idx: udi_index_t,
        ops_idx: udi_index_t,
    },
    ParentBindOps {
        meta_idx: udi_index_t,
        region_idx: udi_index_t,
        ops_idx: udi_index_t,
        bind_cb_idx: udi_index_t,
    },
    InternalBindOps {
        meta_idx: udi_index_t,
        region_idx: udi_index_t,
        primary_ops_idx: udi_index_t,
        secondary_ops_idx: udi_index_t,
        bind_cb_idx: udi_index_t,
    },

    Device {
        device_name: MsgNum,
        meta_idx: udi_index_t,
        attributes: AttributeList<'a>,
    },
    /// A hint to the environment as to what child drivers might be enumerated
    Enumerates {
        device_name: MsgNum,
        min_num: u32,
        max_num: u32,
        meta_idx: udi_index_t,
        attributes: AttributeList<'a>,
    },
    /// Indicates that an instance might bind to multiple parents
    MultiParent,
    Region {
        region_idx: udi_index_t,
        attributes: RegionAttributes,
    },

    // --- Source-only ---
    SourceFiles(&'a str),
    CompileOptions(&'a str),
    SourceRequires(&'a str, Version),
}

#[derive(Debug)]
pub struct Version(u16);
impl ::core::str::FromStr for Version {
    type Err = ::core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if ! s.starts_with("0x") {
            Err(u32::from_str("x").unwrap_err())
        }
        else {
            Ok(Version(u16::from_str_radix(&s[2..], 16)?))
        }
    }
}

#[derive(Debug,PartialEq)]
pub struct MsgNum(pub u16);
impl ::core::str::FromStr for MsgNum {
    type Err = ::core::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MsgNum(u16::from_str_radix(s, 10)?))
    }
}

fn parse_int(v: &str) -> Result<u32, ::std::num::ParseIntError> {
    if v.starts_with("0x") {
        Ok(u32::from_str_radix(&v[2..], 16)?)
    }
    else {
        Ok(v.parse()?)
    }
}

#[derive(Debug,Clone)]
/// A parser for a space-separated list of attributes `<name> <ty> <value>`
pub struct AttributeList<'a>(::core::str::SplitWhitespace<'a>);
impl<'a> AttributeList<'a> {
    pub fn parse_one(&mut self) -> Result<Option<(&'a str, Attribute<'a>)>,String> {

        let Some(name) = self.0.next() else {
            return Ok(None);
            };
        let ty = self.0.next().ok_or("no ty")?;
        let val = self.0.next().ok_or("no val")?;
        let attr = match ty
            {
            "string" => Attribute::String(EscapedStr(val)),
            "ubit32" => Attribute::Ubit32(parse_int(val).map_err(|e| format!("{} {:?}", e, val))?),
            "booleans" => Attribute::Boolean(match val
                {
                "T"|"t" => true,
                "F"|"f" => false,
                _ => return Err(format!("Unknown value for boolean: {:?}", val)),
                }),
            "array" => Attribute::Array8(HexStr(val)),
            _ => return Err(format!("Unknown type {:?}", ty)),
            };
        Ok( Some( (name, attr) ) )
    }
}
impl<'a> Iterator for AttributeList<'a> {
    type Item = (&'a str, Attribute<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_one().expect("Iterating over an invalid AttributeList")
    }
    
}
#[derive(Debug,PartialEq)]
pub enum Attribute<'a> {
    String(EscapedStr<'a>),
    Ubit32(u32),
    Boolean(bool),
    Array8(HexStr<'a>),
}
#[derive(Debug,PartialEq)]
pub struct HexStr<'a>(&'a str);
impl<'a> HexStr<'a> {
    pub fn iter(&self) -> impl Iterator<Item=u8> + 'a {
        self.0.as_bytes().chunks(2)
            .map(move |v| u8::from_str_radix(::core::str::from_utf8(v).unwrap(), 16))
            .map(move |v| v.unwrap_or(0))
    }
}
impl<'a> PartialEq<&'a [u8]> for HexStr<'_> {
    fn eq(&self, other: &&'a [u8]) -> bool {
        *self == **other
    }
}
impl PartialEq<[u8]> for HexStr<'_> {
    fn eq(&self, other: &[u8]) -> bool {
        self.iter().eq(other.iter().copied())
    }
}
#[derive(Debug,PartialEq)]
pub struct EscapedStr<'a>(&'a str);
impl<'a> EscapedStr<'a> {
    pub fn new(v: &str) -> EscapedStr {
        EscapedStr(v)
    }
    pub fn chars(&self) -> impl Iterator<Item=char> + 'a {
        let mut is_escape = false;
        self.0.chars()
            .filter_map(move |c| {
                if is_escape {
                    is_escape = false;
                    Some(match c
                        {
                        '_' => ' ',
                        'H' => '#',
                        '\\' => '\\',
                        'p' => 9 as char,   // Paragraph break, use vertical tab
                        'm' => {
                            todo!("Message indexes!");
                            },
                        _ => '?',
                        })
                }
                else if c == '\\' {
                    is_escape = true;
                    None
                }
                else {
                    Some(c)
                }
            })
    }
}
impl PartialEq<str> for EscapedStr<'_> {
    fn eq(&self, other: &str) -> bool {
        self.chars().eq(other.chars())
    }
}
impl<'a> PartialEq<&'a str> for EscapedStr<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        *self == **other
    }
}

#[derive(Default,Debug)]
pub struct RegionAttributes
{
    pub ty: Option<RegionType>,
    pub binding: Option<RegionBinding>,
    pub priority: Option<RegionPriority>,
    pub latency: Option<RegionLatency>,
    pub overrun_time_ns: Option<u32>,
}
#[derive(Default,Debug)]
pub enum RegionType {
    #[default]
    Normal,
    Fp,
}
impl ::core::str::FromStr for RegionType {
    type Err = &'static str;//Box<dyn ::std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s
            {
            "normal" => RegionType::Normal,
            "fp" => RegionType::Fp,
            _ => return Err("Unknown region type".into()),
            })
    }
}
#[derive(Default,Debug)]
pub enum RegionBinding {
    #[default]
    Static,
    Dynamic,
}
impl ::core::str::FromStr for RegionBinding {
    type Err = Box<dyn ::std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s
            {
            "static" => RegionBinding::Static,
            "dynamic" => RegionBinding::Dynamic,
            _ => return Err("Unknown region binding".into()),
            })
    }
}
#[derive(Default,Debug)]
pub enum RegionPriority {
    Lo,
    #[default]
    Med,
    Hi,
}
impl ::core::str::FromStr for RegionPriority {
    type Err = Box<dyn ::std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s
            {
            "lo" => RegionPriority::Lo,
            "med" => RegionPriority::Med,
            "hi" => RegionPriority::Hi,
            _ => return Err("Unknown region priority".into()),
            })
    }
}
#[derive(Default,Debug)]
pub enum RegionLatency {
    PowerfailWarning,
    Overrunnable,
    Retryable,
    #[default]
    NonOverrunable,
    NonCritial,
}
impl ::core::str::FromStr for RegionLatency {
    type Err = &'static str;//Box<dyn ::std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s
            {
            "powerfail_warning" => RegionLatency::PowerfailWarning,
            "overrunable" => RegionLatency::Overrunnable,
            "retryable" => RegionLatency::Retryable,
            "non_overrunable" => RegionLatency::NonOverrunable,
            "non_critical" => RegionLatency::NonCritial,
            _ => return Err("Unknown region latency".into()),
            })
    }
}

impl<'a> Entry<'a>
{
    pub fn parse_line(line: &'a str) -> Result<Self,Box<dyn ::std::error::Error>>
    {
        fn get_str<'a>(ents: &mut impl Iterator<Item=&'a str>) -> Result<&'a str,Box<dyn ::std::error::Error>> {
            Ok(ents.next().ok_or("Unexpected EOL")?)
        }
        fn get<'a, T: 'a + ::core::str::FromStr>(ents: &mut impl Iterator<Item=&'a str>) -> Result<T,Box<dyn ::std::error::Error>>
        where
            T::Err: Into<Box<dyn ::std::error::Error>>
        {
            Ok(get_str(ents)?.parse().map_err(|e: T::Err| e.into())?)
        }
        #[cfg(false_)]
        fn get_remainder<'a>(ents: &mut ::core::str::SplitWhitespace<'a>) -> Result<&'a str,Box<dyn ::std::error::Error>> {
            let rv = ents.remainder().ok_or("Unexpected EOL")?;
            while let Some(_) = ents.next() {
            }
            rv
        }
        fn get_remainder<'a>(ents: &mut ::core::str::SplitWhitespace<'a>) -> Result<&'a str,Box<dyn ::std::error::Error>> {
            let v = get_str(ents)?;
            let mut end_ptr = v.as_ptr() as usize + v.len();
            while let Some(v) = ents.next() {
                assert!(v.as_ptr() as usize > end_ptr);
                end_ptr = v.as_ptr() as usize + v.len();
            }
            let out_len = end_ptr - v.as_ptr() as usize;
            // SAFE: The pointer is in-bounds, and source data is valid UTF-8
            unsafe {
                Ok(::core::str::from_utf8_unchecked(::core::slice::from_raw_parts(v.as_ptr(), out_len)))
            }
        }
        let mut ents = line.split_whitespace();
        let e1 = ents.next().unwrap();
        let rv = match e1
        {
        "properties_version" => Entry::PropertiesVersion(get(&mut ents)?),
        "supplier" => Entry::Supplier(get(&mut ents)?),
        "contact"  => Entry::Contact(get(&mut ents)?),
        "name"     => Entry::Name(get(&mut ents)?),
        "shortname" => Entry::Shortname(get_str(&mut ents)?),

        "release" => Entry::Release {
            sequence_number: get(&mut ents)?,
            release_string: get_str(&mut ents)?
            },
        "requires" => Entry::Requires(get_str(&mut ents)?, get(&mut ents)?),
        "module" => Entry::Module { filename: get_str(&mut ents)? },
        
        "locale" => Entry::Locale { locale: get_str(&mut ents)? },
        "message" => Entry::Message(get(&mut ents)?, get_remainder(&mut ents)?),
        "disaster_message" => Entry::DiasterMessage(get(&mut ents)?, get_remainder(&mut ents)?),
        "message_file" => Entry::MessageFile(get_str(&mut ents)?),

        // 30.5 Property Declarations for Libraries
        "provides" => {
            let v = Entry::Provides {
                interface_name: get_str(&mut ents)?,
                version_number: get(&mut ents)?
            };
            get_remainder(&mut ents).ok();
            v
            },
        "symbols" => {
            let w1 = get_str(&mut ents)?;
            let (library_symbol, provided_symbol) = match get_str(&mut ents) {
                Ok("as") => (Some(w1), get_str(&mut ents)?,),
                Err(_) => (None, w1,),
                Ok(_) => return Err("Expected `as`".into()),
                };
            Entry::Symbols { library_symbol, provided_symbol }
            },
        "category" => Entry::Category(get(&mut ents)?),

        // 30.6 Property Declarations for Drivers
        "meta" => Entry::Metalang {
            meta_idx: udi_index_t(get(&mut ents)?),
            interface_name: get_str(&mut ents)?,
            },
        "child_bind_ops" => Entry::ChildBindOps {
            meta_idx  : udi_index_t(get(&mut ents)?),
            region_idx: udi_index_t(get(&mut ents)?),
            ops_idx   : udi_index_t(get(&mut ents)?),
        },
        "parent_bind_ops" => Entry::ParentBindOps {
            meta_idx   : udi_index_t(get(&mut ents)?),
            region_idx : udi_index_t(get(&mut ents)?),
            ops_idx    : udi_index_t(get(&mut ents)?),
            bind_cb_idx: udi_index_t(get(&mut ents)?),
        },
        "internal_bind_ops" => Entry::InternalBindOps {
            meta_idx         : udi_index_t(get(&mut ents)?),
            region_idx       : udi_index_t(get(&mut ents)?),
            primary_ops_idx  : udi_index_t(get(&mut ents)?),
            secondary_ops_idx: udi_index_t(get(&mut ents)?),
            bind_cb_idx      : udi_index_t(get(&mut ents)?),
        },
        "device" => Entry::Device {
            device_name: get(&mut ents)?,
            meta_idx   : udi_index_t(get(&mut ents)?),
            attributes: AttributeList(::core::mem::replace(&mut ents, "".split_whitespace())),
        },
        "enumerates" => Entry::Enumerates {
            device_name: get(&mut ents)?,
            min_num: get(&mut ents)?,
            max_num: get(&mut ents)?,
            meta_idx: udi_index_t(get(&mut ents)?),
            attributes: AttributeList(::core::mem::replace(&mut ents, "".split_whitespace())),
        },
        "multi_parent" => Entry::MultiParent,
        "region" => Entry::Region {
            region_idx: udi_index_t(get(&mut ents)?),
            attributes: {
                let mut r = RegionAttributes::default();
                while let Some(region_attribute) = ents.next()
                {
                    fn set_value<T>(dst: &mut Option<T>, v: T) -> Result<(),Box<dyn ::std::error::Error>> {
                        match dst {
                        Some(_) => Err("Double-set of region attribute".into()),
                        None => Ok(*dst = Some(v)),
                        }
                    }
                    match region_attribute
                    {
                    "ty" => set_value(&mut r.ty, get(&mut ents)?)?,
                    "binding" => set_value(&mut r.binding, get(&mut ents)?)?,
                    "priority" => set_value(&mut r.priority, get(&mut ents)?)?,
                    "latency" => set_value(&mut r.latency, get(&mut ents)?)?,
                    "overrun_time" => set_value(&mut r.overrun_time_ns, get(&mut ents)?)?,
                    _ => todo!("Unknown region attribute - {:?}", region_attribute),
                    }
                }
                r
                }
            },
        "readable_file" => todo!(),
        "custom" => todo!(),
        "config_choices" => todo!(),

        // 30.7 Build-Only Properties
        // - These aren't really useful for rust drivers
        "source_files" => Entry::SourceFiles(get_remainder(&mut ents)?),
        "compile_options" => Entry::CompileOptions(get_remainder(&mut ents)?),
        "source_requires" => Entry::SourceRequires(get_str(&mut ents)?, get(&mut ents)?),
        _ => panic!("Unknown statement in `udiprops` - {:?}", e1),
        };
        if let Some(_) = ents.next() {
            return Err("Junk at end of line".into());
        }
        Ok(rv)
    }
}