use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::meta::ParseNestedMeta;

#[derive(Default)]
pub struct Config {
    storage_type: Option<syn::Path>,
    path_prefix: Option<syn::Path>,
}
struct FullConfig {
    storage_type: syn::Path,
    path_prefix: syn::Path,
}

impl Config {
    pub(crate) fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("crate_path") {
            let value = meta.value()?;
            self.path_prefix = Some(value.parse()?);
        } else if meta.path.is_ident("storage_type") {
            let value = meta.value()?;
            self.storage_type = Some(value.parse()?);
        }
        Ok(())
    }

    fn explicit(self) -> syn::Result<FullConfig> {
        let span = Span::call_site();
        let Some(storage_type) = self.storage_type else {
            return Err(syn::Error::new(span, "Error::NoStorageType"));
        };
        let path_prefix = self.path_prefix.unwrap_or(syn::parse_quote!(::bitbybit));
        Ok(FullConfig {
            storage_type,
            path_prefix,
        })
    }
}

pub fn fallback_impl(input: &syn::ItemStruct) -> TokenStream {
    quote! {
        #[derive(Copy, Clone)]
        #input
    }
}

pub fn bitfield(config: Config, input: &syn::ItemStruct) -> syn::Result<TokenStream> {
    let config = config.explicit()?;

    let attrs = &input.attrs;
    let vis = &input.vis;
    let prefix = &config.path_prefix;
    let struct_generics = &input.generics;
    let struct_name = &input.ident;
    let struct_store = &config.storage_type;
    let self_bits = quote!(<#struct_name as #prefix::BitSize<#struct_store>>::BITS);
    let base_bits = quote!(<#struct_store as #prefix::BitSize<#struct_store>>::BITS);

    let fields = input.fields.iter();
    let fields_methods = fields.clone().enumerate().map(|(i, field)| {
        let prev_fields_tys = fields.clone().take(i).map(|f| &f.ty);
        let ty = &field.ty;
        let offset = quote!(0 #(+ <#prev_fields_tys as #prefix::BitSize<#struct_store>>::BITS)*);
        let (name, setter_name) = match &field.ident {
            Some(name) => (name.clone(), format_ident!("with_{name}")),
            None => (format_ident!("get_{i}"), format_ident!("with_{i}")),
        };
        let attrs = &field.attrs;
        let bitsize_trait_turbofish = quote!(<#ty as #prefix::BitSize<#struct_store>>);
        quote! {
            #(#attrs)*
            #[inline(always)]
            pub fn #name(&self) -> #bitsize_trait_turbofish::Unpacked {
                #bitsize_trait_turbofish::from_offset::<{#offset}>(self.0)
            }
            #(#attrs)*
            #[inline(always)]
            pub fn #setter_name(&self, new_value: #ty) -> Self {
                Self(#bitsize_trait_turbofish::with_offset::<{#offset}>(new_value, &self.0))
            }
        }
    });
    let bits = fields.clone().map(|f| {
        let ty = &f.ty;
        quote!(<#ty as #prefix::BitSize<#struct_store>>::BITS)
    });
    let lower_than = format_ident!("{struct_name}CompileChecks");
    Ok(quote! {
        #[doc(hidden)]
        struct #lower_than<const OFFSET: u32>;
        impl <const OFFSET: u32> #lower_than<OFFSET> {
            const CAN_OFFSET: () = assert!(OFFSET + #self_bits <= #base_bits, "Offset is too large");
            const CAN_OFFSET_SIZE: () = assert!(#self_bits <= #base_bits, "Offset is too large");
        }
        impl #struct_generics #prefix::BitSize<#struct_store> for #struct_name {
            type Unpacked = Self;

            const BITS: u32 = 0 #(+ #bits)*;

            #[inline(always)]
            fn from_offset<const OFFSET: u32>(data: #struct_store) -> Self::Unpacked {
                let () = #lower_than::<OFFSET>::CAN_OFFSET;
                Self(data >> OFFSET)
            }
            #[inline(always)]
            fn with_offset<const OFFSET: u32>(self, data: &#struct_store) -> #struct_store {
                // Note: this prevents a const eval error in the `()` branch of `const MASK`
                // The const eval error is good, as it prevents invalid code from compiling, but
                // we would prefer using a more explicit and actionable error message, such as
                // all this absolute nonsense we are writting
                const fn naive_mask(offset: u32) -> u32 { (1 << offset) - 1 }

                const MASK: #struct_store = match () {
                    () if #self_bits == #base_bits => #struct_store::MAX,
                    () if #self_bits > #base_bits => {assert!(#self_bits <= #base_bits, concat!(
                        stringify!(#struct_name),
                        " requires more bits than ",
                        stringify!(#struct_store),
                        " can store.",
                    )); unreachable!()},
                    () => naive_mask(#self_bits) as #struct_store,
                };
                let () = #lower_than::<OFFSET>::CAN_OFFSET;
                // We assume all bits over Self::BITS are always set to 0
                (*data & !(MASK << OFFSET)) | (self.0 << OFFSET)
            }
        }
        #( #attrs )*
        #vis struct #struct_name ( #struct_store );

        impl #struct_generics #struct_name {
            /// Verifies that the struct bit count can be stored in the declared storage type
            /// at compile time.
            #[doc(hidden)]
            const _VALID_BITSIZE: () = assert!(#self_bits <= #base_bits);

            #( #fields_methods )*
        }
    })
}
