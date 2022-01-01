//! Utility library, mainly contains helpful macros

macro_rules! flex_buffer {
    ($lib:expr, $type:ty, $size:expr) => {
        NvFlexAllocBuffer(
            $lib,
            $size,
            std::mem::size_of::<$type>().try_into().unwrap(),
            NvFlexBufferType_eNvFlexBufferHost,
        )
    };
}

/*
let phases: *mut c_int = std::mem::transmute(NvFlexMap(
                       buffers.phases,
                       NvFlexMapFlags_eNvFlexMapWait,
                   )); */

macro_rules! flex_map {
    ($var:expr) => {
        std::mem::transmute(NvFlexMap($var, NvFlexMapFlags_eNvFlexMapWait))
    };
}

pub(crate) use flex_buffer;
pub(crate) use flex_map;
