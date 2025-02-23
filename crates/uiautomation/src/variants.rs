use std::fmt::Display;
use std::mem::ManuallyDrop;
use std::ptr::null_mut;

use windows::Win32::Foundation::BSTR;
use windows::Win32::Foundation::DECIMAL;
use windows::Win32::System::Com::CY;
use windows::Win32::System::Com::IDispatch;
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Com::VARIANT;
use windows::Win32::System::Com::VARIANT_0;
use windows::Win32::System::Com::VARIANT_0_0;
use windows::Win32::System::Com::VARIANT_0_0_0;
use windows::Win32::System::Ole::*;
use windows::core::HRESULT;
use windows::core::HSTRING;
use windows::core::IUnknown;
use windows::core::Interface;
use windows::core::PSTR;

use super::Error;
use super::Result;
use super::errors::ERR_NULL_PTR;
use super::errors::ERR_TYPE;

const VARIANT_TRUE: i16 = -1;
const VARIANT_FALSE: i16 = 0;

/// enum type value for `Variant`
#[derive(Clone, PartialEq)]
pub enum Value {
    EMPTY,
    NULL,
    VOID,
    I1(i8),
    I2(i16),
    I4(i32),
    I8(i64),
    INT(i32),
    UI1(u8),
    UI2(u16),
    UI4(u32),
    UI8(u64),
    UINT(u32),
    R4(f32),
    R8(f64),
    CURRENCY(i64),
    DATE(f64),
    STRING(String),
    UNKNOWN(IUnknown),
    DISPATCH(IDispatch),
    ERROR(HRESULT),
    HRESULT(HRESULT),
    BOOL(bool),
    VARIANT(Variant),
    DECIMAL(DECIMAL),
    SAFEARRAY(SafeArray),
    ARRAY(SafeArray)
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::EMPTY => write!(f, "EMPTY"),
            Value::NULL => write!(f, "NULL"),
            Value::VOID => write!(f, "VOID"),
            Value::I1(value) => write!(f, "I1({})", value),
            Value::I2(value) => write!(f, "I2({})", value),
            Value::I4(value) => write!(f, "I4({})", value),
            Value::I8(value) => write!(f, "I8({})", value),
            Value::INT(value) => write!(f, "INT({})", value),
            Value::UI1(value) => write!(f, "UI1({})", value),
            Value::UI2(value) => write!(f, "UI2({})", value),
            Value::UI4(value) => write!(f, "UI4({})", value),
            Value::UI8(value) => write!(f, "UI8({})", value),
            Value::UINT(value) => write!(f, "UNIT({})", value),
            Value::R4(value) => write!(f, "R4({})", value),
            Value::R8(value) => write!(f, "R8({})", value),
            Value::CURRENCY(value) => write!(f, "CY({})", value),
            Value::DATE(value) => write!(f, "DATE({})", value),
            Value::STRING(value) => write!(f, "STRING({})", value),
            Value::UNKNOWN(_) => write!(f, "UNKNOWN"),
            Value::DISPATCH(_) => write!(f, "DISPATCH"),
            Value::ERROR(value) => write!(f, "ERROR({})", value.0),
            Value::HRESULT(value) => write!(f, "HRESULT({})", value.0),
            Value::BOOL(value) => write!(f, "BOOL({})", value),
            Value::VARIANT(value) => write!(f, "VARIANT({})", value),
            Value::DECIMAL(_) => write!(f, "DECIMAL"),
            Value::SAFEARRAY(value) => write!(f, "SAFEARRAY({})", value),
            Value::ARRAY(value) => write!(f, "ARRAY({})", value),
        }
    }
}

/// A Wrapper for windows `VARIANT`
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Variant {
    value: VARIANT
}

impl Variant {
    /// Create a null variant.
    fn new_null(vt: VARENUM) -> Variant {
        let mut val = VARIANT_0_0::default();
        val.vt = vt.0 as u16;

        let variant = VARIANT {
            Anonymous: VARIANT_0 {
                Anonymous: ManuallyDrop::new(val)
            }
        };

        variant.into()
    }

    /// Create a `Variant` from `vt` and `value`.
    fn new(vt: VARENUM, value: VARIANT_0_0_0) -> Variant {
        let variant = VARIANT {
            Anonymous: VARIANT_0 {
                Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                    vt: vt.0 as u16,
                    wReserved1: 0,
                    wReserved2: 0,
                    wReserved3: 0,
                    Anonymous: value
                })
            }
        };

        variant.into()
    }

    /// Retrieve the variant type as `i32`.
    fn vt(&self) -> i32 {
        unsafe {
            self.value.Anonymous.Anonymous.vt as i32
        }
    }

    /// Retrieve the variant type as `VARENUM`.
    pub fn get_type(&self) -> VARENUM {
        VARENUM(self.vt())
    }

    /// Retrieve the data of the variant.
    pub(crate) unsafe fn get_data(&self) -> &VARIANT_0_0_0 {
        &self.value.Anonymous.Anonymous.Anonymous
    }

    /// Try to get value.
    pub fn get_value(&self) -> Result<Value> {
        self.try_into()
    }

    /// Check whether the variant is null.
    /// 
    /// Return `true` when vt is `VT_EMPTY`, `VT_NULL` or `VT_VOID`.
    pub fn is_null(&self) -> bool {
        let vt = self.vt();
        vt == VT_EMPTY.0 || vt == VT_NULL.0 || vt == VT_VOID.0
    }

    /// Check whether the variant is string.
    /// 
    /// Return `true` when vt is `VT_BSTR`, `VT_LPWSTR` or `VT_LPSTR`.
    pub fn is_string(&self) -> bool {
        let vt = self.vt();
        vt == VT_BSTR.0 || vt == VT_LPWSTR.0 || vt == VT_LPSTR.0
    }

    /// Try to get string value.
    /// 
    /// Return `String` value when vt is `VT_BSTR`, `VT_LPWSTR` or `VT_LPSTR`.
    pub fn get_string(&self) -> Result<String> {
        let value = self.get_value()?;
        match value {
            Value::STRING(str) => Ok(str),
            _ => Err(Error::new(ERR_TYPE, "Error Variant Type"))
        }
    }

    /// Check whether the variant is array.
    /// 
    /// Return `true` when vt is `VT_SAFEARRAY` or `VT_ARRAY`.
    pub fn is_array(&self) -> bool {
        let vt = self.vt();
        vt == VT_SAFEARRAY.0 || vt == VT_ARRAY.0
    }

    /// Try to get array value.
    /// 
    /// Return `SafeArray` value when vt is `VT_SAFEARRAY` or `VT_ARRAY`.
    pub fn get_array(&self) -> Result<SafeArray> {
        let value = self.get_value()?;
        match value {
            Value::SAFEARRAY(arr) => Ok(arr),
            Value::ARRAY(arr) => Ok(arr),
            _ => Err(Error::new(ERR_TYPE, "Error Variant Type"))
        }
    }

    pub fn abs(&self) -> Result<Variant> {
        let v = unsafe {
            VarAbs(&self.value)?
        };

        Ok(v.into())
    }

    pub fn add(&self, augend: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarAdd(&self.value, &augend.value)?
        };

        Ok(v.into())
    }

    pub fn subtract(&self, subtrahend: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarSub(&self.value, &subtrahend.value)?
        };

        Ok(v.into())
    }

    pub fn multiply(&self, multiplicand: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarMul(&self.value, &multiplicand.value)?
        };

        Ok(v.into())
    }

    pub fn divide(&self, divisor: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarDiv(&self.value, &divisor.value)?
        };

        Ok(v.into())
    }

    pub fn mod_by(&self, m: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarMod(&self.value, &m.value)?
        };

        Ok(v.into())
    }

    pub fn negate(&self) -> Result<Variant> {
        let v = unsafe {
            VarNeg(&self.value)?
        };

        Ok(v.into())
    }

    pub fn not(&self) -> Result<Variant> {
        let v = unsafe {
            VarNot(&self.value)?
        };

        Ok(v.into())
    }

    pub fn and(&self, val: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarAnd(&self.value, &val.value)?
        };

        Ok(v.into())
    }

    pub fn or(&self, val: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarOr(&self.value, &val.value)?
        };

        Ok(v.into())
    }

    pub fn xor(&self, val: &Variant) -> Result<Variant> {
        let v = unsafe {
            VarXor(&self.value, &val.value)?
        };

        Ok(v.into())
    }
}

impl From<VARIANT> for Variant {
    fn from(value: VARIANT) -> Self {
        Self {
            value
        }
    }
}

impl Into<VARIANT> for Variant {
    fn into(self) -> VARIANT {
        self.value
    }
}

impl AsRef<VARIANT> for Variant {
    fn as_ref(&self) -> &VARIANT {
        &self.value
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(val) = self.get_value() {
            write!(f, "{}", val)
        } else {
            Err(std::fmt::Error {})
        }
    }
}

impl From<Value> for Variant {
    fn from(value: Value) -> Self {
        match value {
            Value::EMPTY => Variant::new_null(VT_EMPTY),
            Value::NULL => Variant::new_null(VT_NULL),
            Value::VOID => Variant::new_null(VT_VOID),
            Value::I1(v) => Variant::new(VT_I1, VARIANT_0_0_0 { bVal: v as u8 }),
            Value::I2(v) => Variant::new(VT_I2, VARIANT_0_0_0 { iVal: v }),
            Value::I4(v) => Variant::new(VT_I4, VARIANT_0_0_0 { lVal: v }),
            Value::I8(v) => Variant::new(VT_I8, VARIANT_0_0_0 { llVal: v }),
            Value::INT(v) => Variant::new(VT_INT, VARIANT_0_0_0 { lVal: v }),
            Value::UI1(v) => Variant::new(VT_UI1, VARIANT_0_0_0 { bVal: v }),
            Value::UI2(v) => Variant::new(VT_UI2, VARIANT_0_0_0 { uiVal: v }),
            Value::UI4(v) => Variant::new(VT_UI4, VARIANT_0_0_0 { ulVal: v }),
            Value::UI8(v) => Variant::new(VT_UI8, VARIANT_0_0_0 { ullVal: v }),
            Value::UINT(v) => Variant::new(VT_UINT, VARIANT_0_0_0 { uintVal: v }),
            Value::R4(v) => Variant::new(VT_R4, VARIANT_0_0_0 { fltVal: v }),
            Value::R8(v) => Variant::new(VT_R8, VARIANT_0_0_0 { dblVal: v }),
            Value::CURRENCY(v) => Variant::new(VT_CY, VARIANT_0_0_0 { cyVal: CY { int64: v} }),
            Value::DATE(v) => Variant::new(VT_DATE, VARIANT_0_0_0 { date: v }),
            Value::STRING(v) => Variant::new(VT_BSTR, VARIANT_0_0_0 { bstrVal: ManuallyDrop::new(BSTR::from(v)) }),
            Value::UNKNOWN(v) => Variant::new(VT_UNKNOWN, VARIANT_0_0_0 { punkVal: ManuallyDrop::new(Some(v)) }),
            Value::DISPATCH(v) => Variant::new(VT_DISPATCH, VARIANT_0_0_0 { pdispVal: ManuallyDrop::new(Some(v)) }),
            Value::ERROR(v) => Variant::new(VT_ERROR, VARIANT_0_0_0 { intVal: v.0 }),
            Value::HRESULT(v) => Variant::new(VT_HRESULT, VARIANT_0_0_0 { intVal: v.0 }),
            Value::BOOL(v) => Variant::new(VT_BOOL, VARIANT_0_0_0 { boolVal: if v { VARIANT_TRUE } else { VARIANT_FALSE }}),
            Value::VARIANT(mut v) => Variant::new(VT_VARIANT, VARIANT_0_0_0 { pvarVal: &mut v.value }),
            Value::DECIMAL(mut v) => Variant::new(VT_DECIMAL, VARIANT_0_0_0 { pdecVal: &mut v }),
            Value::SAFEARRAY(v) => Variant::new(VT_SAFEARRAY, VARIANT_0_0_0 { parray: v.array }),
            Value::ARRAY(v) => Variant::new(VT_SAFEARRAY, VARIANT_0_0_0 { parray: v.array }),
        }
    }
}

impl TryInto<Value> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<Value> {
        let vt = self.vt();

        if vt == VT_EMPTY.0 {
            Ok(Value::EMPTY)
        } else if vt == VT_NULL.0 {
            Ok(Value::NULL)
        } else if vt == VT_VOID.0 {
            Ok(Value::VOID)
        } else if vt == VT_I1.0 {
            let val = unsafe {
                self.get_data().bVal as i8
            }; 
            Ok(Value::I1(val))
        } else if vt == VT_I2.0 {
            let val = unsafe {
                self.get_data().iVal
            };
            Ok(Value::I2(val))
        } else if vt == VT_I4.0 {
            let val = unsafe {
                self.get_data().lVal
            };
            Ok(Value::I4(val))
        } else if vt == VT_I8.0 {
            let val = unsafe {
                self.get_data().llVal
            };
            Ok(Value::I8(val))
        } else if vt == VT_INT.0 {
            let val = unsafe {
                self.get_data().lVal
            };
            Ok(Value::INT(val))
        } else if vt == VT_UI1.0 {
            let val = unsafe {
                self.get_data().bVal
            };
            Ok(Value::UI1(val))
        } else if vt == VT_UI2.0 {
            let val = unsafe {
                self.get_data().uiVal
            };
            Ok(Value::UI2(val))
        } else if vt == VT_UI4.0 {
            let val = unsafe {
                self.get_data().ulVal
            };
            Ok(Value::UI4(val))
        } else if vt == VT_UI8.0 {
            let val = unsafe {
                self.get_data().ullVal
            };
            Ok(Value::UI8(val))
        } else if vt == VT_UINT.0 {
            let val = unsafe {
                self.get_data().uintVal
            };
            Ok(Value::UINT(val))
        } else if vt == VT_R4.0 {
            let val = unsafe {
                self.get_data().fltVal
            };
            Ok(Value::R4(val))
        } else if vt == VT_R8.0 {
            let val = unsafe {
                self.get_data().dblVal
            };
            Ok(Value::R8(val))
        } else if vt == VT_CY.0 {
            let val = unsafe {
                self.get_data().cyVal.int64
            };
            Ok(Value::CURRENCY(val))
        } else if vt == VT_DATE.0 {
            let val = unsafe {
                self.get_data().date
            };
            Ok(Value::DATE(val))
        } else if vt == VT_BSTR.0 || vt == VT_LPSTR.0 {
            let val = unsafe {
                self.get_data().bstrVal.to_string()
            };
            Ok(Value::STRING(val))
        } else if vt == VT_LPSTR.0 {
            let val = unsafe {
                if self.get_data().pcVal.is_null() {
                    String::from("")
                } else {
                    let lpstr = self.get_data().pcVal.0;
                    let mut end = lpstr;
                    while *end != 0 {
                        end = end.add(1);
                    };
                    String::from_utf8_lossy(std::slice::from_raw_parts(lpstr, end.offset_from(lpstr) as _)).into()
                }
            };

            Ok(Value::STRING(val))
        } else if vt == VT_DISPATCH.0 {
            let val = unsafe {
                if let Some(ref disp) = *self.get_data().ppdispVal {
                    Value::DISPATCH(disp.clone())
                } else {
                    Value::NULL
                }
            };
            Ok(val)
        } else if vt == VT_UNKNOWN.0 {
            let val = unsafe {
                if let Some(ref unkown) = *self.get_data().ppunkVal {
                    Value::UNKNOWN(unkown.clone())
                } else {
                    Value::NULL
                }
            };
            Ok(val)
        } else if vt == VT_ERROR.0 {
            let val = unsafe {
                self.get_data().intVal
            };
            Ok(Value::HRESULT(HRESULT(val)))
        } else if vt == VT_HRESULT.0 {
            let val = unsafe {
                self.get_data().intVal
            };
            Ok(Value::HRESULT(HRESULT(val)))
        } else if vt == VT_BOOL.0 {
            let val = unsafe {
                self.get_data().__OBSOLETE__VARIANT_BOOL != 0
            };
            Ok(Value::BOOL(val))
        } else if vt == VT_VARIANT.0 {
            let val = unsafe {
                (*self.get_data().pvarVal).clone()
            };
            Ok(Value::VARIANT(val.into()))
        } else if vt == VT_DECIMAL.0 {
            let val = unsafe {
                (*self.get_data().pdecVal).clone()
            };
            Ok(Value::DECIMAL(val))
        } else if vt == VT_SAFEARRAY.0 || vt == VT_ARRAY.0 {
            let arr = unsafe {
                self.get_data().parray.clone()
            };
            Ok(Value::SAFEARRAY(SafeArray::new(arr, false)))
        } else {
            Err(Error::new(ERR_TYPE, ""))
        }
    }
}

impl TryInto<Value> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<Value> {
        (&self).try_into()
    }
}

impl From<bool> for Variant {
    fn from(value: bool) -> Self {
        Value::BOOL(value).into()
    }
}

impl TryInto<bool> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<bool> {
        // let vt = self.vt();
        let val: i16 = unsafe {
            match self.get_type() {
                VT_BOOL => self.get_data().boolVal,
                VT_CY => VarBoolFromCy(self.get_data().cyVal)?,
                VT_DATE => VarBoolFromDate(self.get_data().date)?,
                VT_DECIMAL => VarBoolFromDec(self.get_data().pdecVal)?,
                VT_I1 => VarBoolFromI1(self.get_data().cVal)?,
                VT_I2 => VarBoolFromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT => VarBoolFromI4(self.get_data().lVal)?,
                VT_I8 => VarBoolFromI8(self.get_data().llVal)?,
                VT_R4 => VarBoolFromR4(self.get_data().fltVal)?,
                VT_R8 => VarBoolFromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR => {
                    let str = self.get_string()?;
                    let str: HSTRING = str.into();
                    VarBoolFromStr(&str, 0, 0)?
                }, 
                VT_UI1 => VarBoolFromUI1(self.get_data().bVal)?,
                VT_UI2 => VarBoolFromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT => VarBoolFromUI4(self.get_data().ulVal)?,
                VT_UI8 => VarBoolFromUI8(self.get_data().ullVal)?,
                VT_DISPATCH => if let Some(ref disp) = *self.get_data().pdispVal {
                    VarBoolFromDisp(disp, 0)?
                } else {
                    VARIANT_FALSE
                },
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };
        Ok(val != 0)
    }
}

impl TryInto<bool> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<bool> {
        (&self).try_into()
    }
}

impl From<&str> for Variant {
    fn from(value: &str) -> Self {
        Value::STRING(value.into()).into()
    }
}

impl From<String> for Variant {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&String> for Variant {
    fn from(value: &String) -> Self {
        value.as_str().into()
    }
}

impl TryInto<String> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<String> {
        if self.is_string() {
            self.get_string()
        } else {
            // let vt = self.get_type();
            let str: BSTR = unsafe {
                match self.get_type() {
                    VT_BOOL => VarBstrFromBool(self.get_data().boolVal, 0, 0)?,
                    VT_CY => VarBstrFromCy(self.get_data().cyVal, 0, 0)?,
                    VT_DATE => VarBstrFromDate(self.get_data().date, 0, 0)?,
                    VT_DECIMAL => VarBstrFromDec(self.get_data().pdecVal, 0, 0)?,
                    VT_DISPATCH => if let Some(ref disp) = *self.get_data().pdispVal {
                        VarBstrFromDisp(disp, 0, 0)?
                    } else {
                        BSTR::default()
                    },
                    VT_I1 => VarBstrFromI1(self.get_data().cVal, 0, 0)?,
                    VT_I2 => VarBstrFromI2(self.get_data().iVal, 0, 0)?,
                    VT_I4 | VT_INT => VarBstrFromI4(self.get_data().lVal, 0, 0)?,
                    VT_I8 => VarBstrFromI8(self.get_data().llVal, 0, 0)?,
                    VT_R4 => VarBstrFromR4(self.get_data().fltVal, 0, 0)?,
                    VT_R8 => VarBstrFromR8(self.get_data().dblVal, 0, 0)?,
                    VT_UI1 => VarBstrFromUI1(self.get_data().bVal, 0, 0)?,
                    VT_UI2 => VarBstrFromUI2(self.get_data().uiVal, 0, 0)?,
                    VT_UI4 | VT_UINT => VarBstrFromUI4(self.get_data().ulVal, 0, 0)?,
                    VT_UI8 => VarBstrFromUI8(self.get_data().ullVal, 0, 0)?,
                    _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
                }
            };
            Ok(str.to_string())
        }
    }
}

impl TryInto<String> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<String> {
        (&self).try_into()
    }
}

impl From<i8> for Variant {
    fn from(value: i8) -> Self {
        Value::I1(value).into()
    }
}

macro_rules! variant_as_i1 {
    ($func:ident, $value:expr) => {
        {
            let pc = PSTR::null();
            $func($value, pc)?;
            (*pc.0) as i8
        }
    };
}

macro_rules! variant_atoi {
    ($func:ident, $value:expr) => {
        {
            let str = $value;
            let str: HSTRING = str.into();
            $func(&str, 0, 0)?
        }
    };
}

macro_rules! variant_as_type {
    ($f:ident, $T:ty, $value:expr) => {
        {
            let mut v: [$T; 1] = [0 as _];
            $f($value, v.as_mut_ptr())?;
            v[0]
        }
    };
}

macro_rules! dispatch_as_type {
    ($self:ident, $f:ident) => {
        {
            if let Some(ref disp) = *$self.get_data().pdispVal {
                $f(disp, 0)?
            } else {
                0 as _
            }
        }
    };
}

impl TryInto<i8> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<i8> {
        let val: i8 = unsafe {
            match self.get_type() {
                // VT_BOOL => {
                //     let pc = PSTR::default();
                //     VarI1FromBool(self.get_data().iVal, pc)?;
                //     (*pc.0) as i8
                // }
                VT_BOOL     => variant_as_i1!(VarI1FromBool, self.get_data().boolVal),
                VT_CY       => variant_as_i1!(VarI1FromCy, self.get_data().cyVal),
                VT_DATE     => variant_as_i1!(VarI1FromDate, self.get_data().date),
                VT_DECIMAL  => variant_as_i1!(VarI1FromDec, self.get_data().pdecVal),
                VT_DISPATCH => if let Some(ref disp) = *self.get_data().pdispVal {
                    let pc = PSTR::null();
                    VarI1FromDisp(disp, 0, pc)?;
                    *pc.0 as i8
                } else {
                    0i8
                },
                VT_I1   => self.get_data().bVal as i8,
                VT_I2   => variant_as_i1!(VarI1FromI2, self.get_data().iVal),
                VT_I4 | VT_INT  => variant_as_i1!(VarI1FromI4, self.get_data().lVal),
                VT_I8   => variant_as_i1!(VarI1FromI8, self.get_data().llVal),
                VT_R4   => variant_as_i1!(VarI1FromR4, self.get_data().fltVal),
                VT_R8   => variant_as_i1!(VarI1FromR8, self.get_data().dblVal),
                VT_BSTR | VT_LPWSTR | VT_LPSTR => {
                    let str = self.get_string()?;
                    let str: HSTRING = str.into();
                    let pc = PSTR::null();
                    VarI1FromStr(&str, 0, 0, pc)?;
                    (*pc.0) as i8
                },
                VT_UI1  => variant_as_i1!(VarI1FromUI1, self.get_data().bVal),
                VT_UI2  => variant_as_i1!(VarI1FromUI2, self.get_data().uiVal),
                VT_UI4 | VT_UINT => variant_as_i1!(VarI1FromUI4, self.get_data().ulVal),
                VT_UI8  => variant_as_i1!(VarI1FromUI8, self.get_data().ullVal),
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<i8> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<i8> {
        (&self).try_into()
    }
}

impl From<i16> for Variant {
    fn from(value: i16) -> Self {
        Value::I2(value).into()
    }
}

impl TryInto<i16> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<i16> {
        let val: i16 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarI2FromBool(self.get_data().boolVal)?,
                VT_CY       => variant_as_type!(VarI2FromCy, i16, self.get_data().cyVal),
                // VT_CY       => {
                //     let mut v: [i16; 1] = [0];
                //     VarI2FromCy(self.get_data().cyVal, v.as_mut_ptr())?;
                //     v[0]
                // },
                VT_DATE     => VarI2FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarI2FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarI2FromDisp),
                // VT_DISPATCH => if let Some(ref disp) = *self.get_data().pdispVal {
                //     VarI2FromDisp(disp, 0)?
                // } else {
                //     0i16
                // },
                VT_I1       => VarI2FromI1(self.get_data().cVal)?,
                VT_I2       => self.get_data().iVal,
                VT_I4 | VT_INT  => VarI2FromI4(self.get_data().lVal)?,
                VT_I8       => VarI2FromI8(self.get_data().llVal)?,
                VT_R4       => VarI2FromR4(self.get_data().fltVal)?,
                VT_R8       => VarI2FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarI2FromStr, self.get_string()?), //VarI2FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarI2FromUI1(self.get_data().bVal)?,
                VT_UI2      => VarI2FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => VarI2FromUI4(self.get_data().ulVal)?,
                VT_UI8      => VarI2FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }    
        };

        Ok(val)
    }
}

impl TryInto<i16> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<i16> {
        (&self).try_into()
    }
}

impl From<i32> for Variant {
    fn from(value: i32) -> Self {
        Value::I4(value).into()
    }
}

impl TryInto<i32> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<i32> {
        let val: i32 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarI4FromBool(self.get_data().boolVal)?,
                VT_CY       => VarI4FromCy(self.get_data().cyVal)?,
                VT_DATE     => VarI4FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarI4FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarI4FromDisp),
                // VT_DISPATCH => if let Some(ref disp) = *self.get_data().pdispVal {
                //     VarI4FromDisp(disp, 0)?
                // } else {
                //     0i32
                // },
                VT_I1       => VarI4FromI1(self.get_data().cVal)?,
                VT_I2       => VarI4FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => self.get_data().lVal,
                VT_I8       => VarI4FromI8(self.get_data().llVal)?,
                VT_R4       => VarI4FromR4(self.get_data().fltVal)?,
                VT_R8       => VarI4FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarI4FromStr, self.get_string()?), //VarI4FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarI4FromUI1(self.get_data().bVal)?,
                VT_UI2      => VarI4FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => VarI4FromUI4(self.get_data().ulVal)?,
                VT_UI8      => VarI4FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<i32> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<i32> {
        (&self).try_into()
    }
}

impl From<i64> for Variant {
    fn from(value: i64) -> Self {
        Value::I8(value).into()
    }
}

impl TryInto<i64> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<i64> {
        let val: i64 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarI8FromBool(self.get_data().boolVal)?,
                VT_CY       => VarI8FromCy(self.get_data().cyVal)?,
                VT_DATE     => VarI8FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarI8FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarI8FromDisp),
                // VT_DISPATCH => if let Some(ref disp) = *self.get_data().pdispVal {
                //     VarI8FromDisp(disp, 0)?
                // } else {
                //     0i64
                // },
                VT_I1       => VarI8FromI1(self.get_data().cVal)?,
                VT_I2       => VarI8FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => self.get_data().lVal as i64,
                VT_I8       => self.get_data().llVal,
                VT_R4       => VarI8FromR4(self.get_data().fltVal)?,
                VT_R8       => VarI8FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarI8FromStr, self.get_string()?), //VarI8FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarI8FromUI1(self.get_data().bVal)?,
                VT_UI2      => VarI8FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => VarI8FromUI4(self.get_data().ulVal)?,
                VT_UI8      => VarI8FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<i64> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<i64> {
        (&self).try_into()
    }
}

impl From<f32> for Variant {
    fn from(value: f32) -> Self {
        Value::R4(value).into()
    }
}

impl TryInto<f32> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<f32> {
        let val: f32 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarR4FromBool(self.get_data().boolVal)?,
                VT_CY       => variant_as_type!(VarR4FromCy, f32, self.get_data().cyVal),
                // VT_CY       => {
                //     let mut v: [f32; 1] = [f32::default()];
                //     VarR4FromCy(self.get_data().cyVal, v.as_mut_ptr())?;
                //     v[0]
                // },
                VT_DATE     => VarR4FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarR4FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarR4FromDisp),
                // VT_DISPATCH => if let Some(ref disp) = *self.get_data().pdispVal {
                //     VarR4FromDisp(disp, 0)?
                // } else {
                //     0f32
                // },
                VT_I1       => VarR4FromI1(self.get_data().cVal)?,
                VT_I2       => VarR4FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => VarR4FromI4(self.get_data().lVal)?,
                VT_I8       => VarR4FromI8(self.get_data().llVal)?,
                VT_R4       => self.get_data().fltVal,
                VT_R8       => VarR4FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarR4FromStr, self.get_string()?), //VarR4FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarR4FromUI1(self.get_data().bVal)?,
                VT_UI2      => VarR4FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => VarR4FromUI4(self.get_data().ulVal)?,
                VT_UI8      => VarR4FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<f32> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<f32> {
        (&self).try_into()
    }
}

impl From<f64> for Variant {
    fn from(value: f64) -> Self {
        Value::R8(value).into()
    }
}

impl TryInto<f64> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<f64> {
        let val: f64 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarR8FromBool(self.get_data().boolVal)?,
                VT_CY       => variant_as_type!(VarR8FromCy, f64, self.get_data().cyVal),
                VT_DATE     => VarR8FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarR8FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarR8FromDisp),
                VT_I1       => variant_as_type!(VarR8FromI1, f64, self.get_data().cVal),
                VT_I2       => VarR8FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => VarR8FromI4(self.get_data().lVal)?,
                VT_I8       => VarR8FromI8(self.get_data().llVal)?,
                VT_R4       => VarR8FromR4(self.get_data().fltVal)?,
                VT_R8       => self.get_data().dblVal,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarR8FromStr, self.get_string()?), //VarR8FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarR8FromUI1(self.get_data().bVal)?,
                VT_UI2      => VarR8FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => VarR8FromUI4(self.get_data().ulVal)?,
                VT_UI8      => VarR8FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<f64> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<f64> {
        (&self).try_into()
    }
}

impl From<u8> for Variant {
    fn from(value: u8) -> Self {
        Value::UI1(value).into()
    }
}

impl TryInto<u8> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<u8> {
        let val: u8 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarUI1FromBool(self.get_data().boolVal)?,
                VT_CY       => VarUI1FromCy(self.get_data().cyVal)?,
                VT_DATE     => VarUI1FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarUI1FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarUI1FromDisp),
                VT_I1       => VarUI1FromI1(self.get_data().cVal)?,
                VT_I2       => VarUI1FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => VarUI1FromI4(self.get_data().lVal)?,
                VT_I8       => VarUI1FromI8(self.get_data().llVal)?,
                VT_R4       => VarUI1FromR4(self.get_data().fltVal)?,
                VT_R8       => VarUI1FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarUI1FromStr, self.get_string()?), //VarUI1FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => self.get_data().bVal,
                VT_UI2      => VarUI1FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => VarUI1FromUI4(self.get_data().ulVal)?,
                VT_UI8      => VarUI1FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<u8> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<u8> {
        (&self).try_into()
    }
}

impl From<u16> for Variant {
    fn from(value: u16) -> Self {
        Value::UI2(value).into()
    }
}

impl TryInto<u16> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<u16> {
        let val: u16 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarUI2FromBool(self.get_data().boolVal)?,
                VT_CY       => VarUI2FromCy(self.get_data().cyVal)?,
                VT_DATE     => VarUI2FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarUI2FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarUI2FromDisp),
                VT_I1       => VarUI2FromI1(self.get_data().cVal)?,
                VT_I2       => VarUI2FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => VarUI2FromI4(self.get_data().lVal)?,
                VT_I8       => VarUI2FromI8(self.get_data().llVal)?,
                VT_R4       => VarUI2FromR4(self.get_data().fltVal)?,
                VT_R8       => variant_as_type!(VarUI2FromR8, u16, self.get_data().dblVal), // VarUI2FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarUI2FromStr, self.get_string()?), //VarUI2FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarUI2FromUI1(self.get_data().bVal)?,
                VT_UI2      => self.get_data().uiVal,
                VT_UI4 | VT_UINT    => VarUI2FromUI4(self.get_data().ulVal)?,
                VT_UI8      => VarUI2FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<u16> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<u16> {
        (&self).try_into()
    }
}

impl From<u32> for Variant {
    fn from(value: u32) -> Self {
        Value::UI4(value).into()
    }
}

impl TryInto<u32> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<u32> {
        let val: u32 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarUI4FromBool(self.get_data().boolVal)?,
                VT_CY       => VarUI4FromCy(self.get_data().cyVal)?,
                VT_DATE     => VarUI4FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarUI4FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarUI4FromDisp),
                VT_I1       => VarUI4FromI1(self.get_data().cVal)?,
                VT_I2       => VarUI4FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => VarUI4FromI4(self.get_data().lVal)?,
                VT_I8       => VarUI4FromI8(self.get_data().llVal)?,
                VT_R4       => VarUI4FromR4(self.get_data().fltVal)?,
                VT_R8       => VarUI4FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarUI4FromStr, self.get_string()?), //VarUI4FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarUI4FromUI1(self.get_data().bVal)?,
                VT_UI2      => VarUI4FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => self.get_data().ulVal,
                VT_UI8      => VarUI4FromUI8(self.get_data().ullVal)?,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<u32> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<u32> {
        (&self).try_into()
    }
}

impl From<u64> for Variant {
    fn from(value: u64) -> Self {
        Value::UI8(value).into()
    }
}

impl TryInto<u64> for &Variant {
    type Error = Error;

    fn try_into(self) -> Result<u64> {
        let val: u64 = unsafe {
            match self.get_type() {
                VT_BOOL     => VarUI8FromBool(self.get_data().boolVal)?,
                VT_CY       => VarUI8FromCy(self.get_data().cyVal)?,
                VT_DATE     => VarUI8FromDate(self.get_data().date)?,
                VT_DECIMAL  => VarUI8FromDec(self.get_data().pdecVal)?,
                VT_DISPATCH => dispatch_as_type!(self, VarUI8FromDisp),
                VT_I1       => VarUI8FromI1(self.get_data().cVal)?,
                VT_I2       => VarUI8FromI2(self.get_data().iVal)?,
                VT_I4 | VT_INT  => self.get_data().lVal as _,
                VT_I8       => VarUI8FromI8(self.get_data().llVal)?,
                VT_R4       => VarUI8FromR4(self.get_data().fltVal)?,
                VT_R8       => VarUI8FromR8(self.get_data().dblVal)?,
                VT_BSTR | VT_LPWSTR | VT_LPSTR  => variant_atoi!(VarUI8FromStr, self.get_string()?), //VarUI8FromStr(self.get_string()?, 0, 0)?,
                VT_UI1      => VarUI8FromUI1(self.get_data().bVal)?,
                VT_UI2      => VarUI8FromUI2(self.get_data().uiVal)?,
                VT_UI4 | VT_UINT    => VarUI8FromUI4(self.get_data().ulVal)?,
                VT_UI8      => self.get_data().ullVal,
                _ => return Err(Error::new(ERR_TYPE, "Error Variant Type")),
            }
        };

        Ok(val)
    }
}

impl TryInto<u64> for Variant {
    type Error = Error;

    fn try_into(self) -> Result<u64> {
        (&self).try_into()
    }
}

/// A Wrapper for windows `SAFEARRAY`
#[derive(Debug, PartialEq, Eq)]
pub struct SafeArray {
    array: *mut SAFEARRAY,
    owned: bool
}

impl SafeArray {
    /// Create `SafeArray` wrapper. 
    /// 
    /// if the array is from a VARIANT or owned by other object, set `owned` as `false`.
    pub(crate) fn new(array: *mut SAFEARRAY, owned: bool) -> Self {
        Self {
            array,
            owned
        }
    }

    /// Create a vector array.
    pub fn new_vector(var_type: VARENUM, len: u32) -> Result<Self> {
        unsafe {
            let array = SafeArrayCreateVector(var_type.0 as _, 0, len);
            if array.is_null() {
                Err(Error::new(ERR_NULL_PTR, "Create SafeArray Failed"))
            } else {
                Ok(Self {
                    array,
                    owned: true
                })
            }
        }
    }

    /// Retrieve the raw `*mut SAFEARRAY`
    pub fn get_array(&self) -> *mut SAFEARRAY {
        self.array
    }

    pub fn get_var_type(&self) -> Result<VARENUM> {
        let vt = unsafe {
            SafeArrayGetVartype(self.array)?
        };

        Ok(VARENUM(vt as _))     
    }

    pub fn get_dim(&self) -> u32 {
        unsafe {
            SafeArrayGetDim(self.array)
        }
    }

    pub fn get_lower_bound(&self, dimension: u32) -> Result<i32> {
        Ok(unsafe {
            SafeArrayGetLBound(self.array, dimension)?
        })
    }

    pub fn get_upper_bound(&self, dimension: u32) -> Result<i32> {
        Ok(unsafe {
            SafeArrayGetUBound(self.array, dimension)?
        })
    }

    pub fn get_element<T: Default>(&self, index: i32) -> Result<T> {
        let indices: [i32; 1] = [index];
        let mut value = T::default();
        let v_ref: *mut T = &mut value;
        unsafe {
            SafeArrayGetElement(self.array, indices.as_ptr(), v_ref as _)?
        };
        Ok(value)
    }

    pub fn get_interface<T: Interface>(&self, index: i32) -> Result<T> {
        let indices: [i32; 1] = [index];
        let mut result: Option<T> = None;
        let v_ref: *mut Option<T> = &mut result;
        unsafe {
            SafeArrayGetElement(self.array, indices.as_ptr(),v_ref as _)?
        };

        if let Some(value) = result {
            Ok(value)
        } else {
            Err(Error::new(ERR_NULL_PTR, "NULL Interface"))
        }
    }

    pub fn put_element<T>(&mut self, index: i32, value: T) -> Result<()> {
        let indices: [i32; 1] = [index];
        let v_ref: *const T = &value;
        unsafe {
            SafeArrayPutElement(self.array, indices.as_ptr(), v_ref as _)?
        };
        Ok(())
    }

    pub fn into_vector<T: Default>(&self, var_type: VARENUM) -> Result<Vec<T>> {
        if self.get_var_type()? != var_type {
            return Err(Error::new(ERR_TYPE, "Err SafeArray Type"));
        };

        if self.get_dim() != 1 {
            return Err(Error::new(ERR_TYPE, "Err SafeArray Dimension Count"));
        };

        let lower = self.get_lower_bound(1)?;
        let upper = self.get_upper_bound(1)?;

        let mut arr = Vec::with_capacity((upper - lower + 1) as _);
        for i in lower..=upper {
            let v = self.get_element(i)?;
            arr.push(v);
        };

        Ok(arr)
    }

    pub fn into_string_vector(&self) -> Result<Vec<String>> {
        let bstrs: Vec<BSTR> = self.into_vector(VT_BSTR)?;
        let strings: Vec<String> = bstrs.iter().map(|s| s.to_string()).collect();
        Ok(strings)
    }

    pub fn into_interface_vector<T: Interface>(&self) -> Result<Vec<T>> {
        let vt = self.get_var_type()?;
        if vt != VT_UNKNOWN && vt != VT_DISPATCH {
            return Err(Error::new(ERR_TYPE, "Err SafeArray Type"));
        }

        if self.get_dim() != 1 {
            return Err(Error::new(ERR_TYPE, "Err SafeArray Dimension Count"));
        };

        let lower = self.get_lower_bound(1)?;
        let upper = self.get_upper_bound(1)?;

        let mut arr = Vec::with_capacity((upper - lower + 1) as _);
        for i in lower..=upper {
            let v: T = self.get_interface(i)?;
            arr.push(v);
        };

        Ok(arr)
    }

    pub fn from_vector<T: Default>(var_type: VARENUM, src: &Vec<T>) -> Result<SafeArray> {
        let arr = Self::new_vector(var_type, src.len() as _)?;
        for i in 0..src.len() {
            let indices: [i32; 1] = [i as _];
            let v_ref: *const T = &src[i];
            unsafe {
                SafeArrayPutElement(arr.array, indices.as_ptr(), v_ref as _)?
            };
        };
        Ok(arr)
    }

    pub fn from_string_vector<T: AsRef<str>>(src: &Vec<T>) -> Result<SafeArray> {
        let bstrs: Vec<BSTR> = src.iter().map(|s| s.as_ref().into()).collect();
        Self::from_vector(VT_BSTR, &bstrs)
    }
}

impl From<*mut SAFEARRAY> for SafeArray {
    fn from(array: *mut SAFEARRAY) -> Self {
        Self {
            array,
            owned: true
        }
    }
}

macro_rules! fmt_safe_array {
    ($vec_type:ty, $self:ident, $f:ident) => {
        {
            let vals: Result<$vec_type> = $self.try_into();
            if vals.is_err() {
                return Err(std::fmt::Error {});
            }

            let vals = vals.unwrap();
            write!($f, "[")?;
            for (i, v) in vals.iter().enumerate() {
                if i > 0 {
                    write!($f, ", ")?;
                }
                write!($f, "{}", v)?;
            }
            write!($f, "]")
        }
    };
}

impl Display for SafeArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vt = self.get_var_type();
        if vt.is_err() {
            return Err(std::fmt::Error {});
        }

        match vt.unwrap() {
            VT_BOOL => fmt_safe_array!(Vec<bool>, self, f),
            VT_I1 => fmt_safe_array!(Vec<i8>, self, f),
            VT_I2 => fmt_safe_array!(Vec<i16>, self, f),
            VT_I4 | VT_INT => fmt_safe_array!(Vec<i32>, self, f),
            VT_I8 => fmt_safe_array!(Vec<i64>, self, f),
            VT_UI1 => fmt_safe_array!(Vec<u8>, self, f),
            VT_UI2 => fmt_safe_array!(Vec<u16>, self, f),
            VT_UI4 | VT_UINT => fmt_safe_array!(Vec<u32>, self, f),
            VT_UI8 => fmt_safe_array!(Vec<u64>, self, f),
            VT_R4 => fmt_safe_array!(Vec<f32>, self, f),
            VT_R8 => fmt_safe_array!(Vec<f64>, self, f),
            VT_BSTR | VT_LPWSTR => fmt_safe_array!(Vec<String>, self, f),
            _ => Err(core::fmt::Error {})
        }
    }
}

impl Clone for SafeArray {
    fn clone(&self) -> Self {
        let array = if self.owned && !self.array.is_null() {
            unsafe {
                SafeArrayCopy(self.array).unwrap()
            }
        } else {
            self.array
        };

        Self {
            array,
            owned: self.owned
        }
    }
}

impl Drop for SafeArray {
    fn drop(&mut self) {
        if self.owned && !self.array.is_null() {
            unsafe {
                SafeArrayDestroy(self.array).unwrap();
            }
            self.array = null_mut();
        }
    }
}

impl TryFrom<&Vec<i8>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<i8>) -> Result<Self> {
        Self::from_vector(VT_I1, value)
    }
}

impl TryFrom<Vec<i8>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<i8>) -> Result<Self> {
        Self::from_vector(VT_I1, &value)
    }
}

impl TryInto<Vec<i8>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i8>> {
        self.into_vector(VT_I1)
    }
}

impl TryInto<Vec<i8>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i8>> {
        self.into_vector(VT_I1)
    }
}

impl TryFrom<&Vec<i16>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<i16>) -> Result<Self> {
        Self::from_vector(VT_I2, value)
    }
}

impl TryFrom<Vec<i16>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<i16>) -> Result<Self> {
        Self::from_vector(VT_I2, &value)
    }
}

impl TryInto<Vec<i16>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i16>> {
        self.into_vector(VT_I2)
    }
}

impl TryInto<Vec<i16>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i16>> {
        self.into_vector(VT_I2)
    }
}

impl TryFrom<&Vec<i32>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<i32>) -> Result<Self> {
        Self::from_vector(VT_I4, value)
    }
}

impl TryFrom<Vec<i32>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<i32>) -> Result<Self> {
        Self::from_vector(VT_I4, &value)
    }
}

impl TryInto<Vec<i32>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i32>> {
        if self.get_var_type()? == VT_INT {
            self.into_vector(VT_INT)
        } else {
            self.into_vector(VT_I4)
        }
    }
}

impl TryInto<Vec<i32>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i32>> {
        (&self).try_into()
    }
}

impl TryFrom<&Vec<i64>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<i64>) -> Result<Self> {
        Self::from_vector(VT_I8, value)
    }
}

impl TryFrom<Vec<i64>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<i64>) -> Result<Self> {
        Self::from_vector(VT_I8, &value)
    }
}

impl TryInto<Vec<i64>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i64>> {
        self.into_vector(VT_I8)
    }
}

impl TryInto<Vec<i64>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i64>> {
        self.into_vector(VT_I8)
    }
}

impl TryFrom<&Vec<u8>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self> {
        Self::from_vector(VT_UI1, value)
    }
}

impl TryFrom<Vec<u8>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Self::from_vector(VT_UI1, &value)
    }
}

impl TryInto<Vec<u8>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.into_vector(VT_UI1)
    }
}

impl TryInto<Vec<u8>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.into_vector(VT_UI1)
    }
}

impl TryFrom<&Vec<u16>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<u16>) -> Result<Self> {
        Self::from_vector(VT_UI2, value)
    }
}

impl TryFrom<Vec<u16>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<u16>) -> Result<Self> {
        Self::from_vector(VT_UI2, &value)
    }
}

impl TryInto<Vec<u16>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u16>> {
        self.into_vector(VT_UI2)
    }
}

impl TryInto<Vec<u16>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u16>> {
        self.into_vector(VT_UI2)
    }
}

impl TryFrom<&Vec<u32>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<u32>) -> Result<Self> {
        Self::from_vector(VT_UI4, value)
    }
}

impl TryFrom<Vec<u32>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<u32>) -> Result<Self> {
        Self::from_vector(VT_UI4, &value)
    }
}

impl TryInto<Vec<u32>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u32>> {
        if self.get_var_type()? == VT_UINT {
            self.into_vector(VT_UINT)
        } else {
            self.into_vector(VT_UI4)
        }
    }
}

impl TryInto<Vec<u32>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u32>> {
        (&self).try_into()
    }
}

impl TryFrom<&Vec<u64>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<u64>) -> Result<Self> {
        Self::from_vector(VT_UI8, value)
    }
}

impl TryFrom<Vec<u64>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<u64>) -> Result<Self> {
        Self::from_vector(VT_UI8, &value)
    }
}

impl TryInto<Vec<u64>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u64>> {
        self.into_vector(VT_UI8)
    }
}

impl TryInto<Vec<u64>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u64>> {
        self.into_vector(VT_UI8)
    }
}

impl TryFrom<&Vec<f32>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<f32>) -> Result<Self> {
        Self::from_vector(VT_R4, value)
    }
}

impl TryFrom<Vec<f32>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<f32>) -> Result<Self> {
        Self::from_vector(VT_R4, &value)
    }
}

impl TryInto<Vec<f32>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f32>> {
        self.into_vector(VT_R4)
    }
}

impl TryInto<Vec<f32>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f32>> {
        self.into_vector(VT_R4)
    }
}

impl TryFrom<&Vec<f64>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<f64>) -> Result<Self> {
        Self::from_vector(VT_R8, value)
    }
}

impl TryFrom<Vec<f64>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<f64>) -> Result<Self> {
        Self::from_vector(VT_R8, &value)
    }
}

impl TryInto<Vec<f64>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f64>> {
        self.into_vector(VT_R8)
    }
}

impl TryInto<Vec<f64>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f64>> {
        self.into_vector(VT_R8)
    }
}

impl TryFrom<&Vec<&str>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<&str>) -> Result<Self> {
        Self::from_string_vector(value)
    }
}

impl TryFrom<Vec<&str>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<&str>) -> Result<Self> {
        Self::from_string_vector(&value)
    }
}

impl TryFrom<&Vec<&String>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<&String>) -> Result<Self> {
        Self::from_string_vector(value)
    }
}

impl TryFrom<Vec<&String>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<&String>) -> Result<Self> {
        Self::from_string_vector(&value)
    }
}

impl TryFrom<&Vec<String>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<String>) -> Result<Self> {
        Self::from_string_vector(value)
    }
}

impl TryFrom<Vec<String>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<String>) -> Result<Self> {
        Self::from_string_vector(&value)
    }
}

impl TryInto<Vec<String>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<String>> {
        self.into_string_vector()
    }
}

impl TryInto<Vec<String>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<String>> {
        self.into_string_vector()
    }
}

impl TryFrom<&Vec<bool>> for SafeArray {
    type Error = Error;

    fn try_from(value: &Vec<bool>) -> Result<Self> {
        let bools: Vec<i16> = value.iter().map(|b| if *b { VARIANT_TRUE } else { VARIANT_FALSE }).collect();
        Self::from_vector(VT_BOOL, &bools)
    }
}

impl TryFrom<Vec<bool>> for SafeArray {
    type Error = Error;

    fn try_from(value: Vec<bool>) -> Result<Self> {
        (&value).try_into()
    }
}

impl TryInto<Vec<bool>> for &SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<bool>> {
        let bools: Vec<i16> = self.into_vector(VT_BOOL)?;
        Ok(bools.iter().map(|v| *v != 0).collect())
    }
}

impl TryInto<Vec<bool>> for SafeArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<bool>> {
        (&self).try_into()
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::System::Ole::VT_BOOL;

    use crate::variants::SafeArray;
    use crate::variants::Value;
    use crate::variants::Variant;

    #[test]
    fn test_variant_null() {
        let v = Variant::from(Value::NULL);
        assert!(v.is_null());
    }

    #[test]
    fn test_variant_bool() {
        let v: Variant = true.into();
        assert!(v.get_type() == VT_BOOL);

        let b: bool = v.try_into().unwrap();
        assert!(b);

        let val = Variant::from(Value::STRING("true".into()));
        let b_val: bool = val.try_into().unwrap();
        assert!(b_val);
    }

    #[test]
    fn test_variant_string() {
        let s = Variant::from(Value::STRING("Hello".into()));
        assert!(s.is_string());
        assert!(s.get_string().unwrap() == "Hello");
    }

    #[test]
    fn test_safearray_i1() {
        let vals: Vec<i8> = vec![1, 2, 3];
        let arr: SafeArray = vals.try_into().unwrap();

        assert_eq!(arr.get_dim(), 1);
        assert_eq!(arr.get_lower_bound(1).unwrap(), 0);
        assert_eq!(arr.get_upper_bound(1).unwrap(), 2);

        // println!("{}", arr.to_string());
        let vals: Vec<i8> = arr.try_into().unwrap();

        assert_eq!(vals.len(), 3);
        assert_eq!(vals[0], 1);
        assert_eq!(vals[1], 2);
        assert_eq!(vals[2], 3);
    }

    #[test]
    fn test_safearray_bool() {
        let vals = vec![true, false];
        let arr: SafeArray = vals.try_into().unwrap();

        assert_eq!(arr.get_var_type().unwrap(), VT_BOOL);

        assert_eq!(arr.to_string(), "[true, false]");

        let vals: Vec<bool> = arr.try_into().unwrap();
        assert_eq!(vals.len(), 2);
        assert!(vals[0]);
        assert!(!vals[1]);
    }
}
