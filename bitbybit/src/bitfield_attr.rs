//! Parse attribute to `bitfield` fields.

use std::num::NonZeroUsize;

use syn::{meta::ParseNestedMeta, Token};

enum Parsed {
    Something,
    Nothing,
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode {
    Read,
    ReadWrite,
    Write,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct BitfieldConfig {
    range: (usize, usize),
    mode: Option<Mode>,
    stride: Option<NonZeroUsize>,
}

struct ParseInt(usize);

impl syn::parse::Parse for ParseInt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_int = input.parse::<syn::LitInt>()?;
        Ok(Self(lit_int.base10_parse()?))
    }
}

impl BitfieldConfig {
    fn parse_any(&mut self, meta: ParseNestedMeta) -> syn::Result<Parsed> {
        if meta.path.is_ident("r") || meta.path.is_ident("rw") || meta.path.is_ident("w") {
            if let Some(mode) = self.mode {
                return Err(meta.error("Field is already specified as {mode:?}"));
            }
            self.mode = Some(match () {
                () if meta.path.is_ident("r") => Mode::Read,
                () if meta.path.is_ident("rw") => Mode::ReadWrite,
                () if meta.path.is_ident("w") => Mode::Write,
                () => unreachable!(),
            });
            Ok(Parsed::Something)
        } else if meta.path.is_ident("stride") {
            let result1 = meta.input.parse::<Token![:]>();
            let result2 = meta.input.parse::<Token![=]>();
            if result1.is_err() && result2.is_err() {
                return Err(meta.error("'stride' should be followed by '='."));
            }
            let ParseInt(parsed_int) = meta.input.parse()?;
            let Some(nonzero_int) = NonZeroUsize::new(parsed_int) else {
                return Err(meta.error("A stride of 0 is illegal."));
            };
            self.stride = Some(nonzero_int);
            Ok(Parsed::Something)
        } else {
            Ok(Parsed::Nothing)
        }
    }
    pub(crate) fn parse_bit(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let parsed = self.parse_any(meta)?;

        if matches!(parsed, Parsed::Nothing) {
            let ParseInt(parsed_int) = meta.input.parse()?;
            self.range = (parsed_int, parsed_int + 1)
        }
        Ok(())
    }
    pub(crate) fn parse_bits(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let parsed = self.parse_any(meta)?;

        if matches!(parsed, Parsed::Nothing) {
            let ParseInt(start) = meta.input.parse()?;
            let inclusive = match meta.input.parse::<Token![..]>() {
                // [..] is the "exclusive range" token
                Ok(v) => false,
                // [..=] is the "inclusive range" token
                Err(_) => meta.input.parse::<Token![..=]>().map(|_| true)?,
            };
            let ParseInt(mut end) = meta.input.parse()?;
            if inclusive {
                end += 1;
            }
            self.range = (start, end);
        }
        Ok(())
    }
}
