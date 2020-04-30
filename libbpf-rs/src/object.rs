use core::ffi::c_void;
use std::mem;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

use bitflags::bitflags;

use crate::util;
use crate::*;

/// Sets options for opening a [`Object`]
pub struct ObjectBuilder {
    name: String,
    relaxed_maps: bool,
}

impl ObjectBuilder {
    /// Override the generated name that would have been inferred from the constructor.
    pub fn set_name<T: AsRef<str>>(&mut self, name: T) -> &mut Self {
        self.name = name.as_ref().to_string();
        self
    }

    /// Option to parse map definitions non-strictly, allowing extra attributes/data
    pub fn set_relaxed_maps(&mut self, relaxed_maps: bool) -> &mut Self {
        self.relaxed_maps = relaxed_maps;
        self
    }

    fn opts(&mut self, name: *const c_char) -> libbpf_sys::bpf_object_open_opts {
        libbpf_sys::bpf_object_open_opts {
            sz: mem::size_of::<libbpf_sys::bpf_object_open_opts>() as libbpf_sys::size_t,
            object_name: name,
            relaxed_maps: self.relaxed_maps,
            relaxed_core_relocs: false,
            pin_root_path: ptr::null(),
            attach_prog_fd: 0,
            kconfig: ptr::null(),
        }
    }

    pub fn from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<Object> {
        // Convert path to a C style pointer
        let path_str = path.as_ref().to_str().ok_or_else(|| {
            Error::InvalidInput(format!("{} is not valid unicode", path.as_ref().display()))
        })?;
        let path_c = util::str_to_cstring(path_str)?;
        let path_ptr = path_c.as_ptr();

        // Convert name to a C style pointer
        //
        // NB: we must hold onto a CString otherwise our pointer dangles
        let name = util::str_to_cstring(&self.name)?;
        let name_ptr = if !self.name.is_empty() {
            name.as_ptr()
        } else {
            ptr::null()
        };

        let opts = self.opts(name_ptr);

        let obj = unsafe { libbpf_sys::bpf_object__open_file(path_ptr, &opts) };
        if obj.is_null() {
            return Err(Error::Internal("Could not create bpf_object".to_string()));
        }

        let ret = unsafe { libbpf_sys::bpf_object__load(obj) };
        if ret != 0 {
            return Err(Error::Internal("Could not load bpf_object".to_string()));
        }

        Ok(Object::new(obj))
    }

    pub fn from_memory<T: AsRef<str>>(&mut self, name: T, mem: &[u8]) -> Result<Object> {
        // Convert name to a C style pointer
        //
        // NB: we must hold onto a CString otherwise our pointer dangles
        let name = util::str_to_cstring(name.as_ref())?;
        let name_ptr = if !name.to_bytes().is_empty() {
            name.as_ptr()
        } else {
            ptr::null()
        };

        let opts = self.opts(name_ptr);

        let obj = unsafe {
            libbpf_sys::bpf_object__open_mem(
                mem.as_ptr() as *const c_void,
                mem.len() as libbpf_sys::size_t,
                &opts,
            )
        };
        if obj.is_null() {
            return Err(Error::Internal("Could not create bpf_object".to_string()));
        }

        let ret = unsafe { libbpf_sys::bpf_object__load(obj) };
        if ret != 0 {
            return Err(Error::Internal("Could not load bpf_object".to_string()));
        }

        Ok(Object::new(obj))
    }
}

impl Default for ObjectBuilder {
    fn default() -> Self {
        ObjectBuilder {
            name: String::new(),
            relaxed_maps: false,
        }
    }
}

/// Represents a BPF object file. An object may contain zero or more
/// [`Program`]s and [`Map`]s.
pub struct Object {}

impl Object {
    fn new(_ptr: *mut libbpf_sys::bpf_object) -> Self {
        unimplemented!();
    }

    pub fn name(&self) -> &str {
        unimplemented!();
    }

    pub fn map<T: AsRef<str>>(&mut self, _name: T) -> Option<&mut MapBuilder> {
        unimplemented!();
    }

    pub fn prog<T: AsRef<str>>(&mut self, _name: T) -> Option<&mut ProgramBuilder> {
        unimplemented!();
    }
}

/// Represents a parsed but not yet loaded map.
///
/// Some methods require working with raw bytes. You may find libraries such as
/// [`plain`](https://crates.io/crates/plain) helpful.
pub struct MapBuilder {}

impl MapBuilder {
    pub fn set_map_ifindex(&mut self, _idx: u32) -> &mut Self {
        unimplemented!();
    }

    pub fn set_max_entries(&mut self, _entries: u32) -> &mut Self {
        unimplemented!();
    }

    pub fn set_initial_value(&mut self, _data: &[u8]) -> &mut Self {
        unimplemented!();
    }

    pub fn set_numa_node(&mut self, _node: u32) -> &mut Self {
        unimplemented!();
    }

    pub fn set_inner_map_fd(&mut self, _inner: Map) -> &mut Self {
        unimplemented!();
    }

    pub fn set_flags(&mut self, _flags: MapBuilderFlags) -> &mut Self {
        unimplemented!();
    }

    pub fn load(&mut self) -> Result<Map> {
        unimplemented!();
    }
}

#[rustfmt::skip]
bitflags! {
    pub struct MapBuilderFlags: u64 {
	const NO_PREALLOC     = 1;
	const NO_COMMON_LRU   = 1 << 1;
	const NUMA_NODE       = 1 << 2;
	const RDONLY          = 1 << 3;
	const WRONLY          = 1 << 4;
	const STACK_BUILD_ID  = 1 << 5;
	const ZERO_SEED       = 1 << 6;
	const RDONLY_PROG     = 1 << 7;
	const WRONLY_PROG     = 1 << 8;
	const CLONE           = 1 << 9;
	const MMAPABLE        = 1 << 10;
    }
}

/// Represents a created map.
///
/// The kernel ensure the atomicity and safety of operations on a `Map`. Therefore,
/// this handle is safe to clone and pass around between threads. This is essentially a
/// file descriptor.
///
/// Some methods require working with raw bytes. You may find libraries such as
/// [`plain`](https://crates.io/crates/plain) helpful.
#[derive(Clone)]
pub struct Map {}

impl Map {
    pub fn name(&self) -> &str {
        unimplemented!();
    }

    /// Returns a file descriptor to the underlying map.
    pub fn fd(&self) -> i32 {
        unimplemented!();
    }

    pub fn map_type(&self) -> MapType {
        unimplemented!();
    }

    /// Key size in bytes
    pub fn key_size(&self) -> u32 {
        unimplemented!();
    }

    /// Value size in bytes
    pub fn value_size(&self) -> u32 {
        unimplemented!();
    }

    /// Returns map value as `Vec` of `u8`.
    ///
    /// `key` must have exactly [`Map::key_size()`] elements.
    pub fn lookup(&self, _key: &[u8], _flags: MapFlags) -> Result<Option<Vec<u8>>> {
        unimplemented!();
    }

    /// Deletes an element from the map.
    ///
    /// `key` must have exactly [`Map::key_size()`] elements.
    pub fn delete(&mut self, _key: &[u8]) -> Result<()> {
        unimplemented!();
    }

    /// Same as [`Map::lookup()`] except this also deletes the key from the map.
    ///
    /// `key` must have exactly [`Map::key_size()`] elements.
    pub fn lookup_and_delete(&mut self, _key: &[u8], _flags: MapFlags) -> Result<Option<Vec<u8>>> {
        unimplemented!();
    }

    /// Update an element.
    ///
    /// `key` must have exactly [`Map::key_size()`] elements. `value` must have exatly
    /// [`Map::value_size()`] elements.
    pub fn update(&mut self, _key: &[u8], _value: &[u8], _flags: MapFlags) -> Result<()> {
        unimplemented!();
    }
}

#[rustfmt::skip]
bitflags! {
    /// Flags to configure [`Map`] operations.
    pub struct MapFlags: u64 {
	const ANY      = 0;
	const NO_EXIST = 1;
	const EXIST    = 1 << 1;
	const LOCK     = 1 << 2;
    }
}

/// Type of a [`Map`]. Maps to `enum bpf_map_type` in kernel uapi.
#[non_exhaustive]
pub enum MapType {}

/// Represents a parsed but not yet loaded BPF program.
pub struct ProgramBuilder {}

impl ProgramBuilder {
    pub fn set_prog_type(&mut self, _prog_type: ProgramType) -> &mut Self {
        unimplemented!();
    }

    pub fn set_attach_type(&mut self, _attach_type: ProgramAttachType) -> &mut Self {
        unimplemented!();
    }

    pub fn set_ifindex(&mut self, _idx: i32) -> &mut Self {
        unimplemented!();
    }

    // TODO: more flags here:
    // https://github.com/torvalds/linux/blob/master/include/uapi/linux/bpf.h#L267

    pub fn load(&mut self) -> Result<Program> {
        unimplemented!();
    }
}

/// Type of a [`Program`]. Maps to `enum bpf_prog_type` in kernel uapi.
#[non_exhaustive]
pub enum ProgramType {}

/// Attach type of a [`Program`]. Maps to `enum bpf_attach_type` in kernel uapi.
#[non_exhaustive]
pub enum ProgramAttachType {}

/// Represents a loaded [`Program`].
///
/// The kernel ensure the atomicity and safety of operations on a `Program`. Therefore,
/// this handle is safe to clone and pass around between threads. This is essentially a
/// file descriptor.
///
/// If you attempt to attach a `Program` with the wrong attach method, the `attach_*`
/// method will fail with the appropriate error.
#[derive(Clone)]
pub struct Program {}

impl Program {
    pub fn name(&self) -> &str {
        unimplemented!();
    }

    /// Name of the section this `Program` belongs to.
    pub fn section(&self) -> &str {
        unimplemented!();
    }

    pub fn prog_type(&self) -> ProgramType {
        unimplemented!();
    }

    /// Returns a file descriptor to the underlying program.
    pub fn fd(&self) -> i32 {
        unimplemented!();
    }

    pub fn attach_type(&self) -> ProgramAttachType {
        unimplemented!();
    }

    pub fn attach_cgroup(&mut self, _cgroup_fd: i32, _flags: CgroupAttachFlags) -> Result<Link> {
        unimplemented!();
    }

    pub fn attach_perf_event(&mut self, _pfd: i32) -> Result<Link> {
        unimplemented!();
    }
}

#[rustfmt::skip]
bitflags! {
    pub struct CgroupAttachFlags: u64 {
	const ALLOW_OVERRIDE   = 1;
	const ALLOW_MULTI      = 1 << 1;
	const REPLACE          = 1 << 2;
    }
}
