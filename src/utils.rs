pub fn as_void_ptr<T>(mut_ref: &mut T) -> *mut ::core::ffi::c_void {
    mut_ref as *mut T as *mut ::core::ffi::c_void
}
