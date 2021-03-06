use crate::*;

mod buffer;

pub use buffer::*;

#[derive(Debug, Copy, Clone)]
pub struct TextureUpdateData<'a> {
    pub width: i32,
    pub height: i32,
    pub bytes: Option<&'a [u8]>,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureUpdate<'a> {
    data: Option<TextureUpdateData<'a>>,
    max_level: Option<u32>,
    min_filter: Option<gl::TextureMinFilter>,
    mag_filter: Option<gl::TextureMagFilter>,
    wrap_s: Option<gl::TextureWrap>,
    wrap_t: Option<gl::TextureWrap>,
    wrap_r: Option<gl::TextureWrap>,
    max_anisotropy: Option<f32>,
}

impl<'a> TextureUpdate<'a> {
    #[inline]
    pub fn new() -> Self {
        TextureUpdate {
            data: None,
            max_level: None,
            min_filter: None,
            mag_filter: None,
            wrap_s: None,
            wrap_t: None,
            wrap_r: None,
            max_anisotropy: None,
        }
    }

    #[inline]
    pub fn data(mut self, width: i32, height: i32, bytes: Option<&'a [u8]>) -> Self {
        self.data = Some(TextureUpdateData { width, height, bytes });
        self
    }

    #[inline]
    pub fn max_level(mut self, max_level: u32) -> Self {
        self.max_level = Some(max_level);
        self
    }

    #[inline]
    pub fn min_filter(mut self, min_filter: gl::TextureMinFilter) -> Self {
        self.min_filter = Some(min_filter);
        self
    }

    #[inline]
    pub fn mag_filter(mut self, mag_filter: gl::TextureMagFilter) -> Self {
        self.mag_filter = Some(mag_filter);
        self
    }

    #[inline]
    pub fn wrap_s(mut self, wrap_s: gl::TextureWrap) -> Self {
        self.wrap_s = Some(wrap_s);
        self
    }

    #[inline]
    pub fn wrap_t(mut self, wrap_t: gl::TextureWrap) -> Self {
        self.wrap_t = Some(wrap_t);
        self
    }

    #[inline]
    pub fn wrap_r(mut self, wrap_r: gl::TextureWrap) -> Self {
        self.wrap_r = Some(wrap_r);
        self
    }

    #[inline]
    pub fn max_anisotropy(mut self, max_anisotropy: f32) -> Self {
        self.max_anisotropy = Some(max_anisotropy);
        self
    }
}

macro_rules! impl_texture_format {
    ($(($InternalFormat: ident, $Format: ident, $ComponentFormat: ident),)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Copy, Clone)]
        pub enum TextureFormat {
            $(
                $InternalFormat,
            )*
        }

        impl TextureFormat {
            #[inline]
            pub fn internal_format(self) -> gl::InternalFormat {
                match self {
                    $(
                        TextureFormat::$InternalFormat => gl::$InternalFormat.into(),
                    )*
                }
            }

            #[inline]
            pub fn format(self) -> gl::Format {
                match self {
                    $(
                        TextureFormat::$InternalFormat => gl::$Format.into(),
                    )*
                }
            }

            #[inline]
            pub fn component_type(self) -> gl::ComponentFormat {
                match self {
                    $(
                        TextureFormat::$InternalFormat => gl::$ComponentFormat.into(),
                    )*
                }
            }
        }

        $(
            impl From<gl::$InternalFormat> for TextureFormat {
                #[inline]
                fn from(_: gl::$InternalFormat) -> Self {
                    TextureFormat::$InternalFormat
                }
            }
        )*
    };
}

impl_texture_format! {
    (RGBA8, RGBA, UNSIGNED_BYTE),
    (RGB8, RGB, UNSIGNED_BYTE),
    (RG8, RG, UNSIGNED_BYTE),
    (R8, RED, UNSIGNED_BYTE),
    (RG8UI, RG_INTEGER, UNSIGNED_BYTE),
    (DEPTH24_STENCIL8, DEPTH_STENCIL, UNSIGNED_INT_24_8),
    (R11F_G11F_B10F, RGB, FLOAT),
    (RGBA16F, RGBA, FLOAT),
    (RGB16F, RGB, FLOAT),
    (RG16F, RG, FLOAT),
    (R16F, RED, FLOAT),
    (RGBA32F, RGBA, FLOAT),
    (RGB32F, RGB, FLOAT),
    (RG32F, RG, FLOAT),
    (R32F, RED, FLOAT),
}

pub struct Texture<Shape, Format> {
    name: gl::TextureName,
    shape: Shape,
    format: Format,
}

impl<Shape, Format> Texture<Shape, Format> {
    #[inline]
    pub fn name(&self) -> gl::TextureName {
        self.name
    }

    #[inline]
    pub fn drop(self, gl: &gl::Gl) {
        unsafe {
            gl.delete_texture(self.name);
        }
    }
}

impl<Shape, Format> Texture<Shape, Format>
where
    Shape: Copy + Into<gl::TextureTarget>,
    Format: Copy + Into<TextureFormat>,
{
    #[inline]
    pub fn new(gl: &gl::Gl, shape: Shape, format: Format) -> Self {
        unsafe {
            Texture {
                name: gl.create_texture(shape.into()),
                shape,
                format,
            }
        }
    }

    #[inline]
    pub fn update<'a>(&self, gl: &gl::Gl, update: TextureUpdate<'a>) {
        unsafe {
            gl.bind_texture(self.shape.into(), self.name());

            if let Some(TextureUpdateData { width, height, bytes }) = update.data {
                gl.tex_image_2d(
                    self.shape.into(),
                    0,
                    self.format.into().internal_format(),
                    width,
                    height,
                    self.format.into().format(),
                    self.format.into().component_type(),
                    match bytes {
                        Some(slice) => slice.as_ptr() as *const std::ffi::c_void,
                        None => std::ptr::null(),
                    },
                );
            }

            if let Some(max_level) = update.max_level {
                gl.texture_parameteri(self.name, gl::TEXTURE_MAX_LEVEL, max_level);
            }

            if let Some(min_filter) = update.min_filter {
                gl.texture_parameteri(self.name, gl::TEXTURE_MIN_FILTER, min_filter);
            }

            if let Some(mag_filter) = update.mag_filter {
                gl.texture_parameteri(self.name, gl::TEXTURE_MAG_FILTER, mag_filter);
            }

            if let Some(wrap) = update.wrap_s {
                gl.texture_parameteri(self.name, gl::TEXTURE_WRAP_S, wrap);
            }

            if let Some(wrap) = update.wrap_t {
                gl.texture_parameteri(self.name, gl::TEXTURE_WRAP_T, wrap);
            }

            if let Some(wrap) = update.wrap_r {
                gl.texture_parameteri(self.name, gl::TEXTURE_WRAP_R, wrap);
            }

            if let Some(max_anisotropy) = update.max_anisotropy {
                gl.texture_parameterf(self.name, gl::TEXTURE_MAX_ANISOTROPY, max_anisotropy);
            }

            gl.unbind_texture(self.shape.into());
        }
    }
}

pub trait BufferNameExt: Sized {
    fn new(gl: &gl::Gl) -> Self;
}

impl BufferNameExt for gl::BufferName {
    #[inline]
    fn new(gl: &gl::Gl) -> Self {
        unsafe { gl.create_buffer() }
    }
}

#[derive(Debug)]
pub enum ProgramName {
    Unlinked(gl::ProgramName),
    Linked(gl::ProgramName),
}

impl AsRef<gl::ProgramName> for ProgramName {
    fn as_ref(&self) -> &gl::ProgramName {
        match self {
            ProgramName::Unlinked(name) => name,
            ProgramName::Linked(name) => name,
        }
    }
}

impl ProgramName {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe { ProgramName::Unlinked(gl.create_program()) }
    }

    #[inline]
    pub fn attach<I>(&mut self, gl: &gl::Gl, names: I)
    where
        I: IntoIterator,
        I::Item: AsRef<gl::ShaderName>,
    {
        unsafe {
            for name in names.into_iter() {
                gl.attach_shader(*self.as_ref(), *name.as_ref());
            }
        }
    }

    #[inline]
    pub fn link(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.link_program(*self.as_ref());
            let status = gl.get_programiv(*self.as_ref(), gl::LINK_STATUS);
            // Don't panic from here.
            let name = std::ptr::read(self.as_ref());
            std::ptr::write(
                self,
                match status {
                    gl::LinkStatus::Unlinked => ProgramName::Unlinked(name),
                    gl::LinkStatus::Linked => ProgramName::Linked(name),
                },
            );
        }
    }

    #[inline]
    pub fn log(&self, gl: &gl::Gl) -> String {
        unsafe { gl.get_program_info_log(*self.as_ref()) }
    }

    #[inline]
    pub fn is_linked(&self) -> bool {
        match self {
            ProgramName::Linked(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_unlinked(&self) -> bool {
        match self {
            ProgramName::Unlinked(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum ShaderName {
    Uncompiled(gl::ShaderName),
    Compiled(gl::ShaderName),
}

impl AsRef<gl::ShaderName> for ShaderName {
    fn as_ref(&self) -> &gl::ShaderName {
        match self {
            ShaderName::Uncompiled(name) => name,
            ShaderName::Compiled(name) => name,
        }
    }
}

impl ShaderName {
    #[inline]
    pub fn new<K>(gl: &gl::Gl, kind: K) -> Self
    where
        K: Into<gl::ShaderKind>,
    {
        unsafe { ShaderName::Uncompiled(gl.create_shader(kind.into())) }
    }

    #[inline]
    pub fn compile<I>(&mut self, gl: &gl::Gl, sources: I)
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        unsafe {
            gl.shader_source(*self.as_ref(), sources);
            gl.compile_shader(*self.as_ref());
            let status = gl.get_shaderiv(*self.as_ref(), gl::COMPILE_STATUS);
            // Don't panic from here.
            let name = std::ptr::read(self.as_ref());
            std::ptr::write(
                self,
                match status {
                    gl::CompileStatus::Uncompiled => ShaderName::Uncompiled(name),
                    gl::CompileStatus::Compiled => ShaderName::Compiled(name),
                },
            );
        }
    }

    #[inline]
    pub fn log(&self, gl: &gl::Gl) -> String {
        unsafe { gl.get_shader_info_log(*self.as_ref()) }
    }

    #[inline]
    pub fn is_compiled(&self) -> bool {
        match self {
            ShaderName::Compiled(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_uncompiled(&self) -> bool {
        match self {
            ShaderName::Uncompiled(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct DrawCommand {
    pub count: u32,
    pub prim_count: u32,
    pub first_index: u32,
    pub base_vertex: u32,
    pub base_instance: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ComputeCommand {
    pub work_group_x: u32,
    pub work_group_y: u32,
    pub work_group_z: u32,
}

pub mod buffer_usage {
    use super::*;

    pub trait Variant {
        fn value() -> gl::BufferUsage;
    }

    pub enum Static {}
    pub enum Dynamic {}
    pub enum Stream {}

    impl Variant for Static {
        fn value() -> gl::BufferUsage {
            gl::STATIC_DRAW.into()
        }
    }

    impl Variant for Dynamic {
        fn value() -> gl::BufferUsage {
            gl::DYNAMIC_DRAW.into()
        }
    }

    impl Variant for Stream {
        fn value() -> gl::BufferUsage {
            gl::STREAM_DRAW.into()
        }
    }
}

pub struct Buffer<U> {
    name: gl::BufferName,
    byte_capacity: usize,
    usage: std::marker::PhantomData<U>,
}

impl<U> Buffer<U>
where
    U: buffer_usage::Variant,
{
    pub unsafe fn new(gl: &gl::Gl) -> Self {
        Self {
            name: gl.create_buffer(),
            byte_capacity: 0,
            usage: std::marker::PhantomData,
        }
    }

    pub unsafe fn name(&self) -> gl::BufferName {
        self.name
    }

    pub fn byte_capacity(&self) -> usize {
        self.byte_capacity
    }

    pub unsafe fn invalidate(&mut self, gl: &gl::Gl) {
        if self.byte_capacity > 0 {
            // Invalidate buffer using old capacity.
            gl.invalidate_buffer_data(self.name);
        }
    }

    pub unsafe fn ensure_capacity(&mut self, gl: &gl::Gl, mut byte_capacity: usize) {
        if self.byte_capacity >= byte_capacity {
            return;
        }

        if byte_capacity < (self.byte_capacity + self.byte_capacity / 2) {
            byte_capacity = self.byte_capacity + self.byte_capacity / 2;
        }

        byte_capacity = ((byte_capacity + 15) / 16) * 16;

        gl.named_buffer_reserve(self.name, byte_capacity, U::value());
        self.byte_capacity = byte_capacity;
    }

    pub unsafe fn write(&mut self, gl: &gl::Gl, bytes: &[u8]) {
        debug_assert!(bytes.len() <= self.byte_capacity);
        gl.named_buffer_data(self.name, bytes, U::value());
    }

    pub unsafe fn write_at(&mut self, gl: &gl::Gl, bytes: &[u8], offset: usize) {
        debug_assert!(offset + bytes.len() <= self.byte_capacity);
        gl.named_buffer_sub_data(self.name, offset, bytes);
    }

    pub unsafe fn clear_0u32(&mut self, gl: &gl::Gl, byte_count: usize) {
        debug_assert!(byte_count <= self.byte_capacity);
        gl.clear_named_buffer_sub_data(self.name, gl::R32UI, 0, byte_count, gl::RED, gl::UNSIGNED_INT, None);
    }
}

impl<U> AsRef<gl::BufferName> for Buffer<U> {
    fn as_ref(&self) -> &gl::BufferName {
        &self.name
    }
}

pub type StaticBuffer = Buffer<buffer_usage::Static>;
pub type DynamicBuffer = Buffer<buffer_usage::Dynamic>;
pub type StreamBuffer = Buffer<buffer_usage::Stream>;

pub trait GlExt {
    unsafe fn recreate_buffer(&self, name: &mut gl::BufferName);
}

impl GlExt for gl::Gl {
    unsafe fn recreate_buffer(&self, name: &mut gl::BufferName) {
        self.delete_buffer(*name);
        *name = self.create_buffer();
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ring3<T>([T; 3]);

impl<T> Ring3<T> {
    pub fn new<F>(mut f: F) -> Self
    where
        F: FnMut() -> T,
    {
        Self([f(), f(), f()])
    }
}

impl<T> std::ops::Index<usize> for Ring3<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index % self.0.len()]
    }
}

impl<T> std::ops::IndexMut<usize> for Ring3<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index % self.0.len()]
    }
}

pub trait StorageBufferKind {
    fn create_flags() -> gl::BufferStorageFlag;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct StorageBufferKindWO;

impl StorageBufferKind for StorageBufferKindWO {
    fn create_flags() -> gl::BufferStorageFlag {
        gl::BufferStorageFlag::DYNAMIC_STORAGE | gl::BufferStorageFlag::WRITE
    }
}

pub struct StorageBuffer<K> {
    name: gl::BufferName,
    capacity: usize,
    _kind: K,
}

pub type StorageBufferWO = StorageBuffer<StorageBufferKindWO>;

fn make_mult_64_usize(n: usize) -> usize {
    ((n + 63) / 64) * 64
}

impl<K> StorageBuffer<K>
where
    K: Default,
{
    pub unsafe fn new(gl: &gl::Gl) -> Self {
        Self {
            name: gl.create_buffer(),
            capacity: 0,
            _kind: Default::default(),
        }
    }

    pub fn name(&self) -> gl::BufferName {
        self.name
    }
}

impl StorageBufferWO {
    pub unsafe fn write_at(&mut self, gl: &gl::Gl, offset: usize, bytes: &[u8]) {
        debug_assert!(offset + bytes.len() <= self.capacity);
        gl.named_buffer_sub_data(self.name, offset, bytes);
    }
}

impl<K> StorageBuffer<K>
where
    K: StorageBufferKind,
{
    /// Destroys old data.
    pub unsafe fn reconcile(&mut self, gl: &gl::Gl, capacity: usize) {
        let capacity = make_mult_64_usize(if capacity > self.capacity {
            // Grow
            std::cmp::max(capacity, self.capacity + self.capacity / 2)
        } else if (capacity + capacity / 2) < self.capacity {
            // Shrink
            capacity
        } else {
            if self.capacity > 0 {
                gl.invalidate_buffer_data(self.name);
            }
            return;
        });

        if self.capacity > 0 {
            gl.recreate_buffer(&mut self.name);
        }

        if capacity > 0 {
            gl.named_buffer_storage_reserve(self.name, capacity, K::create_flags());
        }

        self.capacity = capacity;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DepthState {
    pub enabled: bool,
    pub function: gl::DepthFunc,
    pub mask: gl::WriteMask,
}

impl DepthState {
    pub unsafe fn apply(&self, gl: &gl::Gl) {
        gl_set(gl, gl::DEPTH_TEST, self.enabled);
        gl.depth_func(self.function);
        gl.depth_mask(self.mask);
    }

    pub unsafe fn reconcile(&mut self, gl: &gl::Gl, state: Self) {
        if self.enabled != state.enabled {
            gl_set(gl,gl::DEPTH_TEST, state.enabled);
            self.enabled = state.enabled;
        }
        if self.function != state.function {
            gl.depth_func(state.function);
            self.function = state.function;
        }
        if self.mask != state.mask {
            gl.depth_mask(state.mask);
            self.mask = state.mask;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ColorState {
    pub mask: [gl::WriteMask; 4],
}

impl ColorState {
    pub unsafe fn apply(&self, gl: &gl::Gl) {
        let [r, g, b, a] = self.mask;
        gl.color_mask(r, g, b, a);
    }

    pub unsafe fn reconcile(&mut self, gl: &gl::Gl, state: Self) {
        if self.mask != state.mask {
            let [r, g, b, a] = state.mask;
            gl.color_mask(r, g, b, a);
            self.mask = state.mask;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BlendState {
    pub enabled: bool,
}

impl BlendState {
    pub unsafe fn apply(&self, gl: &gl::Gl) {
        gl_set(gl, gl::BLEND, self.enabled);
    }

    pub unsafe fn reconcile(&mut self, gl: &gl::Gl, state: Self) {
        if self.enabled != state.enabled {
            gl_set(gl,gl::BLEND, state.enabled);
            self.enabled = state.enabled;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CullState {
    pub enabled: bool,
    pub face: gl::CullFace,
    pub front: gl::FrontFace,
}

unsafe fn gl_set(gl: &gl::Gl, capability: impl Into<gl::Capability>, enabled: bool) {
    if enabled {
        gl.enable(capability.into());
    } else {
        gl.disable(capability.into());
    }
}

impl CullState {
    pub unsafe fn apply(&self, gl: &gl::Gl) {
        gl_set(gl, gl::CULL_FACE, self.enabled);
        gl.cull_face(self.face);
        gl.front_face(self.front);
    }

    pub unsafe fn reconcile(&mut self, gl: &gl::Gl, state: Self) {
        if self.enabled != state.enabled {
            gl_set(gl, gl::CULL_FACE, state.enabled);
            self.enabled = state.enabled;
        }
        if self.face != state.face {
            gl.cull_face(state.face);
            self.face = state.face;
        }
        if self.front != state.front {
            gl.front_face(state.front);
            self.front = state.front;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RenderState {
    pub depth: DepthState,
    pub color: ColorState,
    pub blend: BlendState,
    pub cull: CullState,
}

impl RenderState {
    pub unsafe fn apply(&self, gl: &gl::Gl) {
        self.depth.apply(gl);
        self.color.apply(gl);
        self.blend.apply(gl);
        self.cull.apply(gl);
    }

    pub unsafe fn reconcile(&mut self, gl: &gl::Gl, state: Self) {
        self.depth.reconcile(gl, state.depth);
        self.color.reconcile(gl, state.color);
        self.blend.reconcile(gl, state.blend);
        self.cull.reconcile(gl, state.cull);
    }
}
