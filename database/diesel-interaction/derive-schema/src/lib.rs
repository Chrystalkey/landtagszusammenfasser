extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use syn::DeriveInput;

///
/// This Macro generates Structures and Functions to
/// enable interaction with diesel. Frankly, I do not know
/// why diesel does not provide these themselves, but anyway.
/// For a struct `Structure` corresponding to a `structure` in
/// the schema.rs, the following syntax:
/// ```
/// #[derive(DieselInteraction)]
/// #[schema_table="structure"]
/// pub struct Structure{...}
/// ```
/// the macro generates the implementation of the 
/// ```
/// pub trait DieselInteraction
/// ```
/// Which only makes sense for entities with an id.
/// Compounded primary keys are not supported.
/// What I want in the end is this:
/// ```
/// #[derive(DieselInteraction)]
/// #[schema_table=gesetzesvorhaben]
/// pub struct Gesetzesvorhaben;
///
/// which then generates:
///     1. the members of Gesetzesvorhaben
///     2. the PaginationResult struct
///     3. the struct UpdateGesetzesvorhaben with all members optional
///     4. impl DieselInteraction for Gesetzesvorhaben
/// ```

#[proc_macro_derive(DieselInteraction, attributes(schema_table))]
pub fn whatever(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    implement_macro(&ast)
}
fn is_i32_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty{
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "i32";
        }
    }
    false
}
use quote::quote;
fn implement_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let schema_table_string = {
        let mut result = None;
        for att in ast.attrs.iter() {
            match att {
                syn::Attribute {
                    style: syn::AttrStyle::Outer,
                    meta: syn::Meta::NameValue(syn::MetaNameValue { value, path, .. }),
                    ..
                } => {
                    if path.is_ident("schema_table") {
                        if let syn::Expr::Lit(lit) = value {
                            if let syn::Lit::Str(lit) = &lit.lit {
                                result = Some(lit.value());
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        result.unwrap()
    };
    let update_struct_name = format_ident!("Update{}", name);
    let fields = match &ast.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => {
                panic!("Only named fields are supported by the DieselIntegration Macro")
            }
        },
        _ => {
            panic!("Only structs are supported by the DieselIntegration Macro")
        }
    };
    let optionized_fields = fields
        .iter()
        .filter(|field| {
            !(field.ident.is_some()
                && field.ident.as_ref().unwrap().to_string() == "id"
                && is_i32_type(&field.ty))
        })
        .map(|fields| {
            let name = &fields.ident;
            let ty = &fields.ty;
            quote! {
                pub #name: Option<#ty>
            }
        });
    let filter_query = fields.iter()
    .filter(|field| {
        !(field.ident.is_some()
            && field.ident.as_ref().unwrap().to_string() == "id"
            && is_i32_type(&field.ty))
    })
    .map(|fields| {
        let name = &fields.ident;
        quote! {
            if let Some(ut_val) = &ut.#name {
                query = query.filter(table::#name.eq(ut_val));
            };
        }
    });

    let schema_table = format_ident!("{}", schema_table_string);
    let update_struct = quote! {
        #[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
        #[diesel(table_name=#schema_table)]
        pub struct #update_struct_name #generics {
            #(#optionized_fields),*
        }
    };
    let gen = quote! {
        #update_struct

        impl DieselInteraction<#update_struct_name #generics,
        Connection,
        PaginationResult<Self>> for #name #generics {
            fn create(it: &Self, conn: &mut Connection) -> QueryResult<Self> {
                use crate::schema::#schema_table::dsl::*;
                insert_into(#schema_table).values(it).get_result::<Self>(conn)
            }
            fn update(conn: &mut Connection, id: i32, ut: &#update_struct_name #generics) -> QueryResult<Self> {
                use crate::schema::#schema_table::dsl::*;
                diesel::update(#schema_table.filter(id.eq(id))).set(ut).get_result(conn)
            }
            fn get(conn: &mut Connection, id: i32) -> QueryResult<Self> {
                use crate::schema::#schema_table::dsl::*;
                #schema_table.filter(id.eq(id)).first::<Self>(conn)
            }
            fn matches(conn: &mut Connection, ut: &#update_struct_name #generics) -> QueryResult<Vec<Self>> {
                use crate::schema::#schema_table::dsl as table;
                let mut query = table::#schema_table.into_boxed();
                #(#filter_query)*
                query.load::<Self>(conn)
            }
            fn paginate(conn: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
                use crate::schema::#schema_table::dsl::*;
                let page_size = if page_size < 1 { 1 } else { page_size };
                let total_items = #schema_table.count().get_result(conn)?;
                let items = #schema_table.limit(page_size).offset(page * page_size).load::<Self>(conn)?;
                Ok(PaginationResult {
                    items,
                    total_items,
                    page,
                    page_size,
                    num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
                })
            }
            fn delete(conn: &mut Connection, id: i32) -> QueryResult<usize> {
                use crate::schema::#schema_table::dsl::*;
                diesel::delete(#schema_table.filter(id.eq(id))).execute(conn)
            }
        }
    };
    gen.into()
}
