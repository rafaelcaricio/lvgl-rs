/// A parameter of C functions.
///
/// This struct represents all relevant information we can extract from the C function declaration
/// of a LVGL public interface. We can use this information to do inference for how the parameter
/// should be represented in a safe Rust API.
pub struct CParameter {
    /// The name of the parameter in the C code.
    name: String,

    /// This is the raw representation of the Rust equivalent of the C type.
    c_type: String,

    /// Takes a pointer to a type that is referenced by the LVGL code permanently.
    owned: bool,

    /// The pointer is not marked as `*const` so the referenced object can be mutated.
    mutable: bool,

    /// We need to check if the value is optional in the C code. We need to check
    /// the function comments for this information.
    ///     - "if NULL then"
    ///     - "if not NULL then"
    ///     - "NULL to"
    nullable: bool,
}
