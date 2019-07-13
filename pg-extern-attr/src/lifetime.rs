// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use syn;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::Type;

fn lifetime_to_anon(lifetime: &mut syn::Lifetime) {
    let anon_lifetime = syn::Ident::new("_", lifetime.ident.span());
    lifetime.ident = anon_lifetime;
}

fn sl_lifetime_def(lifetime_def: &mut syn::LifetimeDef) {
    lifetime_to_anon(&mut lifetime_def.lifetime);

    for lifetime in &mut lifetime_def.bounds {
        lifetime_to_anon(lifetime);
    }
}

fn sl_type_param_bound(bound: &mut syn::TypeParamBound) {
    use syn::TypeParamBound::*;
    match bound {
        Trait(ref mut traitBound) => {
            traitBound
                .lifetimes
                .as_mut()
                .map(|bound_lifetimes| for lifetime_def in &mut bound_lifetimes.lifetimes {});

            sl_path(&mut traitBound.path);
        }
        Lifetime(ref mut lifetime) => lifetime_to_anon(lifetime),
    }
}

fn sl_generic_argument(args: &mut syn::GenericArgument) {
    use syn::GenericArgument::*;
    match args {
        Lifetime(ref mut lifetime) => lifetime_to_anon(lifetime),
        Type(ref mut ty) => strip_type(ty),
        Binding(ref mut binding) => strip_type(&mut binding.ty),
        Constraint(ref mut constraint) => {
            for mut bound in &mut constraint.bounds {
                sl_type_param_bound(&mut bound);
            }
        }
        Const(expr) => unimplemented!("Const not supported by pg-extern: {:?}", expr),
    }
}

fn sl_path(path: &mut syn::Path) {
    for p in &mut path.segments {
        use syn::PathArguments::*;

        let path_arguments = &mut p.arguments;

        match path_arguments {
            None => (),
            AngleBracketed(ref mut angleBracketedGenericArguments) => {
                for genericArgument in &mut angleBracketedGenericArguments.args {
                    sl_generic_argument(genericArgument);
                }
            }
            Parenthesized(ref mut parenthesizedGenericArguments) => {
                for ty in &mut parenthesizedGenericArguments.inputs {
                    strip_type(ty);
                }

                strip_return_type(&mut parenthesizedGenericArguments.output);
            }
        }
    }
}

fn sl_type_path(type_path: &mut syn::TypePath) {
    type_path.qself.as_mut().map(|ref mut qself| {
        strip_type(&mut qself.ty);
    });

    sl_path(&mut type_path.path);
}

pub(crate) fn strip_return_type(return_type: &mut syn::ReturnType) {
    use syn::ReturnType::*;
    match return_type {
        Default => (),
        Type(_, ref mut ty) => strip_type(ty),
    }
}

pub(crate) fn strip_type(ty: &mut syn::Type) {
    use syn::Type::*;

    match ty {
        Slice(ref mut typeSlice) => strip_type(&mut typeSlice.elem),
        Array(typeArray) => strip_type(&mut typeArray.elem),
        Ptr(typePtr) => strip_type(&mut typePtr.elem),
        Reference(typeReference) => strip_type(&mut typeReference.elem),
        BareFn(typeBareFn) => unimplemented!("BareFn not supported by pg-extern: {:?}", typeBareFn),
        Never(typeNever) => (),
        Tuple(typeTuple) => {
            for mut i in &mut typeTuple.elems {
                strip_type(&mut i);
            }
        }
        Path(ref mut typePath) => sl_type_path(typePath),
        TraitObject(typeTraitObject) => unimplemented!(
            "TraitObject not supported by pg-extern: {:?}",
            typeTraitObject
        ),
        ImplTrait(typeImplTrait) => {
            unimplemented!("ImplTrait not supported by pg-extern: {:?}", typeImplTrait)
        }
        Paren(typeParen) => unimplemented!("Paren not supported by pg-extern: {:?}", typeParen),
        Group(typeGroup) => unimplemented!("Group not supported by pg-extern: {:?}", typeGroup),
        Infer(typeInfer) => unimplemented!("Infer not supported by pg-extern: {:?}", typeInfer),
        Macro(typeMacro) => unimplemented!("Macro not supported by pg-extern: {:?}", typeMacro),
        Verbatim(typeVerbatim) => {
            unimplemented!("Verbatim not supported by pg-extern: {:?}", typeVerbatim)
        }
    }
}
