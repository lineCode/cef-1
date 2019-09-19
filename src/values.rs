use cef_sys::{cef_list_value_t, cef_list_value_create, cef_dictionary_value_t, cef_dictionary_value_create, cef_binary_value_t, cef_binary_value_create, cef_value_t, cef_value_create, cef_value_type_t, cef_string_userfree_utf16_free};
use std::convert::{TryInto, TryFrom};

use crate::{
    string::{CefString, CefStringList},
};

#[derive(Debug, Eq, PartialEq)]
#[repr(i32)]
enum ValueType {
    Invalid = cef_value_type_t::VTYPE_INVALID as i32,
    Null = cef_value_type_t::VTYPE_NULL as i32,
    Bool = cef_value_type_t::VTYPE_BOOL as i32,
    Int = cef_value_type_t::VTYPE_INT as i32,
    Double = cef_value_type_t::VTYPE_DOUBLE as i32,
    String = cef_value_type_t::VTYPE_STRING as i32,
    Binary = cef_value_type_t::VTYPE_BINARY as i32,
    Dictionary = cef_value_type_t::VTYPE_DICTIONARY as i32,
    List = cef_value_type_t::VTYPE_LIST as i32,
}

pub struct Value(*mut cef_value_t);

unsafe impl Sync for Value {}
unsafe impl Send for Value {}

impl Value {
    pub fn new() -> Self {
        Self(unsafe { cef_value_create() })
    }
    /// Returns true if the underlying data is valid. This will always be true
    /// for simple types. For complex types (binary, dictionary and list) the
    /// underlying data may become invalid if owned by another object (e.g. list or
    /// dictionary) and that other object is then modified or destroyed. This value
    /// object can be re-used by calling `set_*()` even if the underlying data is
    /// invalid.
    pub fn is_valid(&self) -> bool {
        self.as_ref().is_valid.and_then(|is_valid| Some(unsafe { is_valid(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub fn is_owned(&self) -> bool {
        self.as_ref().is_owned.and_then(|is_owned| Some(unsafe { is_owned(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the underlying data is read-only. Some APIs may expose
    /// read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.as_ref().is_read_only.and_then(|is_read_only| Some(unsafe { is_read_only(self.0) != 0 })).unwrap_or(true)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data. If true modifications to this object will also affect `that`
    /// object and vice-versa.
    pub fn is_same(&self, that: &Value) -> bool {
        self.as_ref().is_same.and_then(|is_same| Some(unsafe { is_same(self.0, that.0) != 0 })).unwrap_or(false)
    }
    /// Returns the underlying value type.
    pub fn get_type(&self) -> ValueType {
        self.as_ref().get_type.and_then(|get_type| {
            Some(match get_type(self.0) {
                cef_value_type_t::VTYPE_INVALID => ValueType::Invalid,
                cef_value_type_t::VTYPE_NULL => ValueType::Null,
                cef_value_type_t::VTYPE_BOOL => ValueType::Bool,
                cef_value_type_t::VTYPE_INT => ValueType::Int,
                cef_value_type_t::VTYPE_DOUBLE => ValueType::Double,
                cef_value_type_t::VTYPE_STRING => ValueType::String,
                cef_value_type_t::VTYPE_BINARY => ValueType::Binary,
                cef_value_type_t::VTYPE_DICTIONARY => ValueType::Dictionary,
                cef_value_type_t::VTYPE_LIST => ValueType::List,
            })
        }).unwrap_or(ValueType::Invalid)
    }
    /// Returns the underlying value as type bool.
    pub fn to_bool(&self) -> bool {
        self.as_ref().get_bool.and_then(|get_bool| Some(unsafe { get_bool(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns the underlying value as type int.
    pub fn to_int(&self) -> i32 {
        self.as_ref().get_int.and_then(|get_int| Some(unsafe { get_int(self.0) as i32 })).unwrap_or(0)
    }
    /// Returns the underlying value as type double.
    pub fn to_double(&self) -> f64 {
        self.as_ref().get_double.and_then(|get_double| Some(unsafe { get_double(self.0) })).unwrap_or(0.0)
    }
    /// Returns the underlying value as type string.
    pub fn to_string(&self) -> String {
        self.as_ref().get_string.and_then(|get_string| {
            let s = unsafe { get_string(self.0) };
            let result = CefString::copy_raw_to_string(s);
            unsafe { cef_string_userfree_utf16_free(s as *mut _); }
            result
        }).unwrap_or_else(|| String::new())
    }
    /// Returns the underlying value as type binary. The returned reference may
    /// become invalid if the value is owned by another object or if ownership is
    /// transferred to another object in the future. To maintain a reference to the
    /// value after assigning ownership to a dictionary or list pass this object to
    /// the [set_value()] function instead of passing the returned reference to
    /// [set_binary()].
    pub fn try_to_binary(&self) -> Option<BinaryValue> {
        self.as_ref().get_binary.and_then(|get_binary| unsafe { get_binary(self.0) }.as_ref().and_then(|binary| Some(BinaryValue(binary, 0))))
    }
    /// Returns the underlying value as type dictionary. The returned reference may
    /// become invalid if the value is owned by another object or if ownership is
    /// transferred to another object in the future. To maintain a reference to the
    /// value after assigning ownership to a dictionary or list pass this object to
    /// the [set_value()] function instead of passing the returned reference to
    /// [set_dictionary()].
    pub fn try_to_dictionary(&self) -> Option<DictionaryValue> {
        self.as_ref().get_dictionary.and_then(|get_dictionary| unsafe { get_dictionary(self.0) }.as_ref().and_then(|dictionary| Some(DictionaryValue(dictionary))))
    }
    /// Returns the underlying value as type list. The returned reference may
    /// become invalid if the value is owned by another object or if ownership is
    /// transferred to another object in the future. To maintain a reference to the
    /// value after assigning ownership to a dictionary or list pass this object to
    /// the [set_value()] function instead of passing the returned reference to
    /// [set_list()].
    pub fn try_to_list(&self) -> Option<ListValue> {
        self.as_ref().get_list.and_then(|get_list| unsafe { get_list(self.0) }.get_ref().and_then(|list| Some(ListValue(list))))
    }
    /// Sets the underlying value as type null. Returns true if the value was
    /// set successfully.
    pub fn set_null(&mut self) -> bool {
        self.as_ref().set_null.and_then(|set_null| Some(unsafe { set_null(self.0) != 0 })).unwrap_or(false)
    }
    /// Sets the underlying value as type bool. Returns true if the value was
    /// set successfully.
    pub fn set_bool(&mut self, value: bool) -> bool {
        self.as_ref().set_bool.and_then(|set_bool| Some(unsafe { set_bool(self.0, if value { 1 } else { 0 }) != 0 })).unwrap_or(false)
    }
    /// Sets the underlying value as type int. Returns true if the value was
    /// set successfully.
    pub fn set_int(&mut self, value: i32) -> bool {
        self.as_ref().set_int.and_then(|set_int| Some(unsafe { set_int(self.0, value as std::os::raw::c_int) != 0 })).unwrap_or(false)
    }
    /// Sets the underlying value as type double. Returns true if the value was
    /// set successfully.
    pub fn set_double(&mut self, value: f64) -> bool {
        self.as_ref().set_double.and_then(|set_double| Some(unsafe { set_double(self.0, value) != 0 })).unwrap_or(false)
    }
    /// Sets the underlying value as type string. Returns true if the value was
    /// set successfully.
    pub fn set_string(&mut self, value: &str) -> bool {
        self.as_ref().set_string.and_then(|set_string| {
            Some(unsafe { set_string(self.0, *CefString::new(value).as_ref()) != 0 })
        }).unwrap_or(false)
    }
    /// Sets the underlying value as type binary. Returns true if the value was
    /// set successfully. This object keeps a reference to |value| and ownership of
    /// the underlying data remains unchanged.
    pub fn set_binary(&mut self, value: &BinaryValue) -> bool {
        self.as_ref().set_binary.and_then(|set_binary| Some(unsafe { set_binary(self.0, *value.as_ref()) != 0 })).unwrap_or(false)
    }
    /// Sets the underlying value as type dict. Returns true if the value was
    /// set successfully. This object keeps a reference to `value` and ownership of
    /// the underlying data remains unchanged.
    pub fn set_dictionary(&mut self, value: &DictionaryValue) -> bool {
        self.as_ref().set_dictionary.and_then(|set_dictionary| Some(unsafe { set_dictionary(self.0, *value.as_ref()) != 0 })).unwrap_or(false)
    }
    /// Sets the underlying value as type list. Returns true if the value was
    /// set successfully. This object keeps a reference to `value` and ownership of
    /// the underlying data remains unchanged.
    pub fn set_list(&mut self, value: &ListValue) -> bool {
        self.as_ref().set_list.and_then(|set_list| Some(unsafe { set_list(self.0, *value.as_ref()) != 0 })).unwrap_or(false)
    }
}

impl From<*mut cef_value_t> for Value {
    fn from(value: *mut cef_value_t) -> Self {
        unsafe { ((*value).base.add_ref.unwrap())(&mut (*value).base); }
        Self(value)
    }
}

impl std::convert::AsRef<cef_value_t> for Value {
    fn as_ref(&self) -> &cef_value_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl PartialEq for Value {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.as_ref().is_equal.and_then(|is_equal| Some(unsafe { is_equal(self.0, that.0) != 0 })).unwrap_or(false)
    }
}

impl Clone for Value {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn clone(&self) -> Self {
        Self(unsafe { (self.as_ref().copy.unwrap())(self.0) })
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe { (self.as_ref().base.release.unwrap())(&mut (*self.0).base); }
    }
}

#[derive(Eq)]
pub struct BinaryValue(*mut cef_binary_value_t, usize);

unsafe impl Sync for BinaryValue {}
unsafe impl Send for BinaryValue {}

impl BinaryValue {
    // Creates a new object that is not owned by any other object. The specified
    // `data` will be copied.
    pub fn new(data: &[u8]) -> Self {
        Self(unsafe { cef_binary_value_create(data.as_ptr() as *const std::os::raw::c_void, data.len()) }, 0)
    }
    /// Returns true if this object is valid. This object may become invalid if
    /// the underlying data is owned by another object (e.g. list or dictionary)
    /// and that other object is then modified or destroyed. Do not call any other
    /// functions if this function returns false.
    pub fn is_valid(&self) -> bool {
        self.as_ref().is_valid.and_then(|is_valid| Some(unsafe { is_valid(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub fn is_owned(&self) -> bool {
        self.as_ref().is_owned.and_then(|is_owned| Some(unsafe { is_owned(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data.
    pub fn is_same(&self, that: &Value) -> bool {
        self.as_ref().is_same.and_then(|is_same| Some(unsafe { is_same(self.0, that.0) != 0 })).unwrap_or(false)
    }
    // Returns the data size.
    pub fn len(&self) -> usize {
        self.as_ref().get_size.and_then(|get_size| Some(unsafe { get_size(self.0) })).unwrap_or(0)
    }
}

impl std::convert::AsRef<cef_binary_value_t> for BinaryValue {
    fn as_ref(&self) -> &cef_binary_value_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl std::io::Read for BinaryValue {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.as_ref().get_data.and_then(|get_data| Some(unsafe { get_data(self.0, buf.as_mut_ptr(), buf.len(), self.1) })).and_then(|result| {
            self.1 += result;
            Some(result)
        }).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "cef_binary_value_t is invalid"))
    }
}

impl std::io::Seek for BinaryValue {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.1 = match pos {
            std::io::SeekFrom::Start(offset) => usize::try_from(offset)?,
            std::io::SeekFrom::Current(offset) => usize::try_from(self.1 as u64 + offset)?,
            std::io::SeekFrom::End(offset) => {
                if offset > self.1 as u64 {
                    0
                } else {
                    (self.1 as u64 - offset) as usize
                }
            },
        };
        Ok(self.1 as u64)
    }
}

impl PartialEq for BinaryValue {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.as_ref().is_equal.and_then(|is_equal| Some(unsafe { is_equal(self.0, that.0) != 0 })).unwrap_or(false)
    }
}

impl Clone for BinaryValue {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn clone(&self) -> Self {
        Self(unsafe { (self.as_ref().copy.unwrap())(self.0) })
    }
}

impl Drop for BinaryValue {
    fn drop(&mut self) {
        unsafe { (self.as_ref().base.release.unwrap())(&mut (*self.0).base); }
    }
}

pub struct DictionaryValue(*mut cef_dictionary_value_t);

unsafe impl Sync for DictionaryValue {}
unsafe impl Send for DictionaryValue {}

impl DictionaryValue {
    pub fn new() -> Self {
        Self(unsafe { cef_dictionary_value_create() })
    }
    /// Returns true if this object is valid. This object may become invalid if
    /// the underlying data is owned by another object (e.g. list or dictionary)
    /// and that other object is then modified or destroyed. Do not call any other
    /// functions if this function returns false.
    pub fn is_valid(&self) -> bool {
        self.as_ref().is_valid.and_then(|is_valid| Some(unsafe { is_valid(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub fn is_owned(&self) -> bool {
        self.as_ref().is_owned.and_then(|is_owned| Some(unsafe { is_owned(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the underlying data is read-only. Some APIs may expose
    /// read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.as_ref().is_read_only.and_then(|is_read_only| Some(unsafe { is_read_only(self.0) != 0 })).unwrap_or(true)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data.
    pub fn is_same(&self, that: &Value) -> bool {
        self.as_ref().is_same.and_then(|is_same| Some(unsafe { is_same(self.0, that.0) != 0 })).unwrap_or(false)
    }
    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.as_ref().get_size.and_then(|get_size| Some(unsafe { get_size(self.0) })).unwrap_or(0)
    }
    /// Removes all values. Returns true on success.
    pub fn clear(&mut self) -> bool {
        self.as_ref().clear.and_then(|clear| Some(unsafe { clear(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the current dictionary has a value for the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.as_ref().has_key.and_then(|has_key| Some(unsafe { has_key(self.0, CefString::new(key).as_ref()) != 0 })).unwrap_or(false)
    }
    /// Reads all keys for this dictionary into the specified vector.
    pub fn keys(&self) -> Vec<String> {
        self.as_ref().get_keys.and_then(|get_keys| {
            let list = CefStringList::new();
            if unsafe { get_keys(self.0, list.get()) } != 0 {
                Some(list.into())
            } else {
                None
            }
        }).unwrap_or(vec![])
    }
    /// Removes the value at the specified key. Returns true if the value
    /// is removed successfully.
    pub fn remove(&mut self, key: &str) -> bool {
        self.as_ref().remove.and_then(|remove| Some(unsafe { remove(self.0, CefString::new(key).as_ref()) != 0 })).unwrap_or(false)
    }
    /// Returns the value type for the specified key.
    pub fn get_type(&self, key: &str) -> ValueType {
        self.as_ref().get_type.and_then(|get_type| Some(match get_type(self.0, CefString::new(key).as_ref()) {
            cef_value_type_t::VTYPE_INVALID => ValueType::Invalid,
            cef_value_type_t::VTYPE_NULL => ValueType::Null,
            cef_value_type_t::VTYPE_BOOL => ValueType::Bool,
            cef_value_type_t::VTYPE_INT => ValueType::Int,
            cef_value_type_t::VTYPE_DOUBLE => ValueType::Double,
            cef_value_type_t::VTYPE_STRING => ValueType::String,
            cef_value_type_t::VTYPE_BINARY => ValueType::Binary,
            cef_value_type_t::VTYPE_DICTIONARY => ValueType::Dictionary,
            cef_value_type_t::VTYPE_LIST => ValueType::List,
        })).unwrap_or(ValueType::Invalid)
    }
    /// Returns the value at the specified `key` as type bool.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.as_ref().get_bool.and_then(|get_bool| Some(unsafe { get_bool(self.0, CefString::new(key).as_ref()) != 0 })).unwrap_or(false)
    }
    /// Returns the value at the specified `key` as type int.
    pub fn get_int(&self, key: &str) -> i32 {
        self.as_ref().get_int.and_then(|get_int| Some(unsafe { get_int(self.0, CefString::new(key).as_ref()) as i32 })).unwrap_or(0)
    }
    /// Returns the value at the specified `key` as type double.
    pub fn get_double(&self, key: &str) -> f64 {
        self.as_ref().get_double.and_then(|get_double| Some(unsafe { get_double(self.0) })).unwrap_or(0.0)
    }
    /// Returns the value at the specified `key` as type string.
    pub fn get_string(&self, key: &str) -> String {
        self.as_ref().get_string.and_then(|get_string| {
            let s = unsafe { get_string(self.0, CefString::new(key).as_ref()) };
            let result = CefString::copy_raw_to_string(s);
            unsafe { cef_string_userfree_utf16_free(s as *mut _); }
            result
        }).unwrap_or_else(|| String::new())
    }
    /// Returns the value at the specified key as type binary. The returned value
    /// will reference existing data.
    pub fn try_get_binary(&self, key: &str) -> Option<BinaryValue> {
        self.as_ref().get_binary.and_then(|get_binary| unsafe { get_binary(self.0, CefString::new(key).as_ref()) }.as_ref().and_then(|binary| Some(BinaryValue(binary, 0))))
    }
    /// Returns the value at the specified key as type dictionary. The returned
    /// value will reference existing data and modifications to the value will
    /// modify this object.
    pub fn try_get_dictionary(&self, key: &str) -> Option<DictionaryValue> {
        self.as_ref().get_dictionary.and_then(|get_dictionary| unsafe { get_dictionary(self.0) }.as_ref().and_then(|dictionary| Some(DictionaryValue(dictionary))))
    }
    /// Returns the value at the specified key as type list. The returned value
    /// will reference existing data and modifications to the value will modify
    /// this object.
    pub fn try_get_list(&self) -> Option<ListValue> {
        self.as_ref().get_list.and_then(|get_list| unsafe { get_list(self.0) }.as_ref().and_then(|list| Some(ListValue(list))))
    }
    /// Sets the value at the specified key. Returns true if the value was set
    /// successfully. If `value` represents simple data then the underlying data
    /// will be copied and modifications to `value` will not modify this object. If
    /// `value` represents complex data (binary, dictionary or list) then the
    /// underlying data will be referenced and modifications to `value` will modify
    /// this object.
    pub fn insert(&mut self, key: &str, value: Value) -> bool {
        self.as_ref().set_value.and_then(|set_value| unsafe { set_value(self.0, CefString::new(key).as_ref(), value.0) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type null. Returns true if the
    /// value was set successfully.
    pub fn insert_null(&mut self, key: &str) -> bool {
        self.as_ref().set_null.and_then(|set_null| unsafe { set_null(self.0, CefString::new(key).as_ref()) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type bool. Returns true if the
    /// value was set successfully.
    pub fn insert_bool(&mut self, key: &str, value: bool) -> bool {
        self.as_ref().set_bool.and_then(|set_bool| unsafe { set_bool(self.0, CefString::new(key).as_ref(), if value { 1 } else { 0 }) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type int. Returns true if the
    /// value was set successfully.
    pub fn insert_int(&mut self, key: &str, value: i32) -> bool {
        self.as_ref().set_int.and_then(|set_int| unsafe { set_int(self.0, CefString::new(key).as_ref(), value) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type double. Returns true if the
    /// value was set successfully.
    pub fn insert_double(&mut self, key: &str, value: f64) -> bool {
        self.as_ref().set_double.and_then(|set_double| unsafe { set_double(self.0, CefString::new(key).as_ref(), value) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type string. Returns true if the
    /// value was set successfully.
    pub fn insert_string(&mut self, key: &str, value: &str) -> bool {
        self.as_ref().set_string.and_then(|set_string| unsafe { set_double(self.0, CefString::new(key).as_ref(), CefString::new(value).as_ref()) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type binary. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn insert_binary(&mut self, key: &str, value: BinaryValue) -> bool {
        self.as_ref().set_binary.and_then(|set_binary| unsafe { set_binary(self.0, CefString::new(key).as_ref(), value.0) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type dict. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn insert_dictionary(&mut self, key: &str, value: DictionaryValue) -> bool {
        self.as_ref().set_dictionary.and_then(|set_dictionary| unsafe { set_dictionary(self.0, CefString::new(key).as_ref(), value.0) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified key as type list. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn insert_list(&mut self, key: &str, value: ListValue) -> bool {
        self.as_ref().set_list.and_then(|set_list| unsafe { set_list(self.0, CefString::new(key).as_ref(), value.0) != 0 }).unwrap_or(false)
    }
}

impl std::convert::AsRef<cef_dictionary_value_t> for DictionaryValue {
    fn as_ref(&self) -> &cef_dictionary_value_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl PartialEq for DictionaryValue {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.as_ref().is_equal.and_then(|is_equal| Some(unsafe { is_equal(self.0, that.0) != 0 })).unwrap_or(false)
    }
}

impl Clone for DictionaryValue {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn clone(&self) -> Self {
        Self(unsafe { (self.as_ref().copy.unwrap())(self.0, 0) })
    }
}

impl Drop for DictionaryValue {
    fn drop(&mut self) {
        unsafe { (self.as_ref().base.release.unwrap())(&mut (*self.0).base); }
    }
}

pub struct ListValue(*mut cef_list_value_t);

unsafe impl Sync for ListValue {}
unsafe impl Send for ListValue {}

impl ListValue {
    pub fn new() -> Self {
        Self(unsafe { cef_list_value_create() })
    }
    /// Returns true if this object is valid. This object may become invalid if
    /// the underlying data is owned by another object (e.g. list or dictionary)
    /// and that other object is then modified or destroyed. Do not call any other
    /// functions if this function returns false.
    pub fn is_valid(&self) -> bool {
        self.as_ref().is_valid.and_then(|is_valid| Some(unsafe { is_valid(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub fn is_owned(&self) -> bool {
        self.as_ref().is_owned.and_then(|is_owned| Some(unsafe { is_owned(self.0) != 0 })).unwrap_or(false)
    }
    /// Returns true if the underlying data is read-only. Some APIs may expose
    /// read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.as_ref().is_read_only.and_then(|is_read_only| Some(unsafe { is_read_only(self.0) != 0 })).unwrap_or(true)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data.
    pub fn is_same(&self, that: &Value) -> bool {
        self.as_ref().is_same.and_then(|is_same| Some(unsafe { is_same(self.0, that.0) != 0 })).unwrap_or(false)
    }
    /// Sets the number of values. If the number of values is expanded all new
    /// value slots will default to type None. Returns true on success.
    pub fn set_len(&mut self, size: usize) -> bool {
        self.as_ref().set_size.and_then(|set_size| Some(unsafe { set_size(self.0, size) != 0 }))
    }
    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.as_ref().get_size.and_then(|get_size| Some(unsafe { get_size(self.0) })).unwrap_or(0)
    }
    /// Removes all values. Returns true on success.
    pub fn clear(&mut self) -> bool {
        self.as_ref().clear.and_then(|clear| Some(unsafe { clear(self.0) != 0 })).unwrap_or(false)
    }
    /// Removes the value at the specified index.
    pub fn remove(&mut self, index: usize) -> bool {
        self.as_ref().remove.and_then(|remove| Some(unsafe { remove(self.0, index) != 0 }))
    }
    /// Returns the value type at the specified index.
    pub fn get_type(&self, index: usize) -> ValueType {
        self.as_ref().get_type.and_then(|get_type| {
            Some(match get_type(self.0, index) {
                cef_value_type_t::VTYPE_INVALID => ValueType::Invalid,
                cef_value_type_t::VTYPE_NULL => ValueType::Null,
                cef_value_type_t::VTYPE_BOOL => ValueType::Bool,
                cef_value_type_t::VTYPE_INT => ValueType::Int,
                cef_value_type_t::VTYPE_DOUBLE => ValueType::Double,
                cef_value_type_t::VTYPE_STRING => ValueType::String,
                cef_value_type_t::VTYPE_BINARY => ValueType::Binary,
                cef_value_type_t::VTYPE_DICTIONARY => ValueType::Dictionary,
                cef_value_type_t::VTYPE_LIST => ValueType::List,
            })
        }).unwrap_or(ValueType::Invalid)
    }
    /// Returns the value at the specified index. For simple types the returned
    /// value will copy existing data and modifications to the value will not
    /// modify this object. For complex types (binary, dictionary and list) the
    /// returned value will reference existing data and modifications to the value
    /// will modify this object.
    pub fn try_get_value(&self, index: usize) -> Option<Value> {
        self.as_ref().get_value.and_then(|get_value| Some(unsafe { get_value(self.0, index) }.as_ref().and_then(|value| Value(value))))
    }
    /// Returns the value at the specified index as type bool.
    pub fn get_bool(&self, index: usize) -> bool {
        self.as_ref().get_bool.and_then(|get_bool| Some(unsafe { get_bool(self.0, index) != 0 }))
    }
    /// Returns the value at the specified index as type int.
    pub fn get_int(&self, index: usize) -> i32 {
        self.as_ref().get_int.and_then(|get_int| Some(unsafe { get_int(self.0, index) as i32 }))
    }
    /// Returns the value at the specified index as type double.
    pub fn get_double(&self, index: usize) -> f64 {
        self.as_ref().get_double.and_then(|get_double| Some(unsafe { get_double(self.0, index) }))
    }
    /// Returns the value at the specified index as type string.
    pub fn get_string(&self, index: usize) -> String {
        self.as_ref().get_string.and_then(|get_string| {
            let s = unsafe { get_string(self.0, index) };
            let result = CefString::copy_raw_to_string(s);
            unsafe { cef_string_userfree_utf16_free(s); }
            result
        }).unwrap_or_else(|| String::new())
    }
    /// Returns the value at the specified index as type binary. The returned value
    /// will reference existing data.
    pub fn try_get_binary(&self, index: usize) -> BinaryValue {
        self.as_ref().get_binary.and_then(|get_binary| Some(unsafe { get_binary(self.0, index) }.as_ref().and_then(|binary| BinaryValue(binary, 0))))
    }
    /// Returns the value at the specified index as type dictionary. The returned
    /// value will reference existing data and modifications to the value will
    /// modify this object.
    pub fn try_get_dictionary(&self, index: usize) -> DictionaryValue {
        self.as_ref().get_dictionary.and_then(|get_dictionary| Some(unsafe { get_dictionary(self.0, index) }.as_ref().and_then(|dictionary| DictionaryValue(dictionary))))
    }
    /// Returns the value at the specified index as type list. The returned value
    /// will reference existing data and modifications to the value will modify
    /// this object.
    pub fn try_get_list(&self, index: usize) -> ListValue {
        self.as_ref().get_list.and_then(|get_list| Some(unsafe { get_list(self.0, index) }.as_ref().and_then(|list| ListValue(list))))
    }
    /// Sets the value at the specified index. Returns true if the value was
    /// set successfully. If `value` represents simple data then the underlying
    /// data will be copied and modifications to `value` will not modify this
    /// object. If `value` represents complex data (binary, dictionary or list)
    /// then the underlying data will be referenced and modifications to `value`
    /// will modify this object.
    pub fn set_value(&mut self, index: usize, value: Value) -> bool {
        self.as_ref().set_value.and_then(|set_value| Some(unsafe { set_value(self.0, index, value.0) != 0 }))
    }
    /// Sets the value at the specified index as type null. Returns true if the
    /// value was set successfully.
    pub fn set_null(&mut self, index: usize) -> bool {
        self.as_ref().set_null.and_then(|set_null| Some(unsafe { set_null(self.0, index) != 0 }))
    }
    /// Sets the value at the specified index as type bool. Returns true if the
    /// value was set successfully.
    pub fn set_bool(&mut self, index: usize, value: bool) -> bool {
        self.as_ref().set_bool.and_then(|set_bool| Some(unsafe { set_bool(self.0, index, if value { 1 } else { 0 }) != 0 }))
    }
    /// Sets the value at the specified index as type int. Returns true if the
    /// value was set successfully.
    pub fn set_int(&mut self, index: usize, value: i32) -> bool {
        self.as_ref().set_int.and_then(|set_int| Some(unsafe { set_int(self.0, index, value) != 0 }))
    }
    /// Sets the value at the specified index as type double. Returns true if the
    /// value was set successfully.
    pub fn set_double(&mut self, index: usize, value: f64) -> bool {
        self.as_ref().set_double.and_then(|set_double| Some(unsafe { set_double(self.0, index, value) != 0 }))
    }
    /// Sets the value at the specified index as type string. Returns true if the
    /// value was set successfully.
    pub fn set_string(&mut self, index: usize, value: &str) -> bool {
        self.as_ref().set_string.and_then(|set_string| Some(unsafe { set_string(self.0, index, CefString::new(value).as_ref()) != 0 }))
    }
    /// Sets the value at the specified index as type binary. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn set_binary(&mut self, index: usize, value: BinaryValue) -> bool {
        self.as_ref().set_binary.and_then(|set_binary| unsafe { set_binary(self.0, index, value.0) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified index as type dict. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn set_dictionary(&mut self, index: usize, value: DictionaryValue) -> bool {
        self.as_ref().set_dictionary.and_then(|set_dictionary| unsafe { set_dictionary(self.0, index, value.0) != 0 }).unwrap_or(false)
    }
    /// Sets the value at the specified index as type list. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn set_list(&mut self, index: usize, value: ListValue) -> bool {
        self.as_ref().set_list.and_then(|set_list| unsafe { set_list(self.0, index, value.0) != 0 }).unwrap_or(false)
    }
}

impl std::convert::AsRef<cef_list_value_t> for ListValue {
    fn as_ref(&self) -> &cef_list_value_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl PartialEq for ListValue {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.as_ref().is_equal.and_then(|is_equal| Some(unsafe { is_equal(self.0, that.0) != 0 })).unwrap_or(false)
    }
}

impl Clone for ListValue {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn clone(&self) -> Self {
        Self(unsafe { (self.as_ref().copy.unwrap())(self.0, 0) })
    }
}

impl Drop for ListValue {
    fn drop(&mut self) {
        unsafe { (self.as_ref().base.release.unwrap())(&mut (*self.0).base); }
    }
}
