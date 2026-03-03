use syn::{GenericArgument, Type, TypePath};

pub fn vec_inner_type(ty: &Type) -> Option<Type> {
	let Type::Path(TypePath { path, .. }) = ty else {
		return None;
	};

	let segment = path.segments.last()?;
	if segment.ident != "Vec" {
		return None;
	}

	let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
		return None;
	};

	let arg = args.args.first()?;

	let GenericArgument::Type(inner) = arg else {
		return None;
	};

	Some(inner.clone())
}

pub fn is_string_type(ty: &Type) -> bool {
	matches!(
		ty,
		Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "String")
	)
}
