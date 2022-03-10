pub mod error;

use error::Error;

use abi_stable::{
    StableAbi,
    sabi_trait,
    package_version_strings,
    library::{LibraryError, RootModule},
    std_types::{
        RString,
        ROption,
        RVec,
        RResult,
        RBox,
        Tuple2,
    },
    sabi_types::VersionStrings,
};
use std::path::Path;


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "ViowPlugin_Ref")))]
#[sabi(missing_field(panic))]
pub struct ViowPlugin {
    #[sabi(last_prefix_field)]
    pub get_name: extern "C" fn() -> RString,

    pub get_loader: extern "C" fn() -> ROption<FiletypeLoader_Ref>,
}

impl RootModule for ViowPlugin_Ref {
    abi_stable::declare_root_module_statics!{ViowPlugin_Ref}

    const BASE_NAME: &'static str = "viow_plugin";
    const NAME: &'static str = "viow_plugin";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}


pub fn load_root_module_in_directory(directory: &Path) -> Result<ViowPlugin_Ref, LibraryError> {
    ViowPlugin_Ref::load_from_directory(directory)
}



#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "FiletypeLoader_Ref")))]
#[sabi(missing_field(panic))]
pub struct FiletypeLoader {
    pub open: extern "C" fn(path: &RString, cycle_time_fs: u64) -> RResult<WaveLoadType, Error>,

    #[sabi(last_prefix_field)]
    pub get_suffix: extern "C" fn() -> RString,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct WaveSignal {
    pub name: RString,
    pub format: RString,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct WaveData {
    pub cycle_start: u64,
    pub cycle_end: u64,
    pub bitranges: RVec<Tuple2<usize, usize>>,
    pub bytes_per_frame: usize,
    pub data: RVec<u8>,
}

impl WaveData {
    pub fn new<'a>(signals: impl Iterator<Item = &'a SignalType>, cycle_range: std::ops::Range<u64>) -> Self {
        let mut ptr = 0;
        let mut next_ptr;
        let mut bitranges = Vec::new();
        
        for signal in signals {
            match signal {
                SignalType::Bit => {
                    next_ptr = ptr + 1;
                }

                SignalType::Vector(a, b) => {
                    let sz = (b - a).abs() as usize;
                    next_ptr = ptr + sz;
                }
            }

            bitranges.push(Tuple2::from((ptr, next_ptr)));
            ptr = next_ptr;
        }

        let bytes_per_frame = ptr;
        let data = vec![0u8; bytes_per_frame * (cycle_range.end - cycle_range.start) as usize];

        Self {
            cycle_start: cycle_range.start,
            cycle_end: cycle_range.end,
            bitranges: bitranges.into(),
            bytes_per_frame,
            data: data.into(),
        }
    }

    pub fn get(&self, signal: u64, cycle: u64) -> Vec<bool> {
        let bitrange = self.bitranges[signal as usize].into_tuple();
        let num_bits = bitrange.1 - bitrange.0;
        let offset = cycle as usize * self.bytes_per_frame;
        let mut rv = Vec::with_capacity(num_bits);

        for bitpos in bitrange.0 .. bitrange.1 {
            let bit = (self.data[offset + bitpos] & 1) as u32;
            rv.push(bit != 0);
        }

        rv
    }

    pub fn set(&mut self, signal: u64, cycle: u64, value: &[bool]) {
        let bitrange = self.bitranges[signal as usize].into_tuple();
        let offset = cycle as usize * self.bytes_per_frame;

        for (i, bitpos) in (bitrange.0 .. bitrange.1).enumerate() {
            self.data[offset + bitpos] = value[i] as u8;
        }
    }
}

#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
pub enum SignalType {
    Bit,
    Vector(i32, i32),
}

#[repr(C)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
pub struct SignalSpec {
    pub name: RString,
    pub typespec: SignalType,
}


#[sabi_trait]
pub trait WaveLoad {
    fn init_signals(&mut self) -> RResult<RVec<SignalSpec>, Error>;
    fn count_cycles(&mut self) -> RResult<u64, Error>;
    #[sabi(last_prefix_field)]
    fn load(&mut self, signals: &RVec<RString>, cycle_range: Tuple2<u64, u64>) -> RResult<WaveData, Error>;
    //fn extract<'a>(&self, data: &WaveData, signal: u64, cycle: u64) -> RVec<u32>;
}

pub type WaveLoadType = WaveLoad_TO<'static, RBox<()>>;
