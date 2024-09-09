extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(DieselInteraction, attributes(connection_type))]
pub fn whatever(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    implement_macro(&ast)
}
fn is_i32_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "i32";
        }
    }
    false
}
fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option" && !segment.arguments.is_empty();
        }
    }
    false
}

fn get_schema_table(ast: &DeriveInput) -> syn::Path {
    let mut tbl_nm = None;
    for att in ast.attrs.iter() {
        match att {
            syn::Attribute {
                style: syn::AttrStyle::Outer,
                meta: syn::Meta::List(syn::MetaList{ path, ..}),
                ..
            } => {
                if path.is_ident("diesel") {
                    let _ = att.parse_nested_meta(
                        |meta|{
                            if meta.path.is_ident("table_name"){
                                let value = meta.value().unwrap();
                                tbl_nm = Some(value.parse::<syn::Path>().unwrap());
                                return Ok(());
                            }
                            Ok(())
                        }
                    );
                }
            }
            _ => {}
        }
    }
    return tbl_nm.expect("No attribute `table_name` specified");
}

fn get_connection_type(ast: &DeriveInput) -> syn::Path {
    for att in ast.attrs.iter() {
        match att {
            syn::Attribute {
                style: syn::AttrStyle::Outer,
                meta: syn::Meta::List(syn::MetaList{ path, ..}),
                ..
            } => {
                if path.is_ident("connection_type") {
                    let table_name = att.parse_args::<syn::Path>().unwrap();
                    return table_name;
                }
            }
            _ => {}
        }
    }
    panic!("No attribute `connection_type` specified")
}

fn implement_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let slug_name = format_ident!("{}", name.to_string().to_lowercase());
    let generics = &ast.generics;
    let schema_table =  get_schema_table(ast);
    let connection_type = get_connection_type(ast);

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
    let substruct_fields = fields
        .iter()
        .filter(|field| {
            !(field.ident.is_some()
                && field.ident.as_ref().unwrap().to_string() == "id"
                && is_i32_type(&field.ty))
        })
        .map(|fields| {
            let name = &fields.ident;
            let ty = &fields.ty;
            let is_opt = is_option(ty);
            
            (
                if is_opt {
                    quote!{
                        pub #name: #ty
                    }
                }else{
                    quote! {
                        pub #name: Option<#ty>
                    }
                },
                quote! {
                    pub #name: #ty
                },
                quote! {
                    if let Some(ut_val) = &ut.#name {
                        query = query.filter(module::#name.eq(ut_val));
                    };
                },
                if is_opt{
                    quote! {
                        #name: row.#name
                    }
                }else{
                    quote! {
                        #name: Some(row.#name)
                    }
                },
                quote! {
                    #name: row.#name
                },
            )
        });
    let update_fields = substruct_fields.clone().map(|(x, _, _, _, _)| x);
    let insert_fields = substruct_fields.clone().map(|(_, x, _, _, _)| x);
    let query_filter_expression = substruct_fields.clone().map(|(_, _, x, _, _)| x);
    let upd_from_fields = substruct_fields.clone().map(|(_, _, _, x, _)| x);
    let insert_from_fields = substruct_fields.clone().map(|(_, _, _, _, x)| x);

    let update_struct = quote! {
        #[derive(Debug, Default, Clone, AsChangeset)]
        #[diesel(table_name=#schema_table)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        pub struct Update #generics {
            #(#update_fields),*
        }
    };
    let insert_struct = quote! {
        #[derive(Clone, Debug, Insertable)]
        #[diesel(table_name=#schema_table)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        pub struct Insert #generics{
            #(#insert_fields),*
        }
    };

    let gen = quote! {
        pub mod #slug_name{
            pub use #schema_table::dsl as module;   // put the dsl module into scope
            pub use #schema_table::table;           // put the table into scope
            use diesel::*;
            use diesel_interaction::{DieselInteractionError, PaginationResult};
            
            pub type Master = super::#name;
            type Connection = #connection_type;
            type Result<T> = std::result::Result<T, DieselInteractionError>;

            #update_struct

            #insert_struct
            impl From<super::#name> for Update{
                fn from(row: super::#name) -> Self{
                    Update{
                        #(#upd_from_fields),*
                    }
                }
            }
            impl From<super::#name> for Insert{
                fn from(row: super::#name) -> Self{
                    Insert{
                        #(#insert_from_fields),*
                    }
                }
            }

            pub async fn insert(conn: &mut Connection, it: Insert) -> Result<i32> {
                let result = conn
                    .interact(
                        move |conn| 
                        diesel::insert_into(table).values(&it)
                        .returning(module::id)
                        .get_result(conn)
                )
                    .await??;
                Ok(result)
            }
            pub async fn update(conn: &mut Connection, id: i32, ut: &Update) -> Result<usize> {
                let utcl = ut.clone();
                let result = conn
                    .interact(move |conn| {
                        diesel::update(table.filter(module::id.eq(id)))
                            .set(utcl)
                            .execute(conn)
                    })
                    .await??;
                Ok(result)
            }
            pub async fn select(conn: &mut Connection, id: i32) -> Result<super::#name> {
                let result = conn
                    .interact(move |conn| {
                        table
                            .filter(module::id.eq(id))
                            .select(super::#name::as_select())
                            .get_result(conn)
                    })
                    .await??;
                Ok(result)
            }
            pub async fn select_matching(conn: &mut Connection, ut: Update) -> Result<Vec<super::#name>> {
                let result = conn
                    .interact(move |conn| {
                        let mut query = table.into_boxed();
                        #(#query_filter_expression)*
                        query.load::<super::#name>(conn)
                    })
                    .await??;
                Ok(result)
            }
            pub async fn paginate(
                conn: &mut Connection,
                page: i64,
                page_size: i64,
            ) -> Result<PaginationResult<super::#name>> {
                let page_size = if page_size < 1 { 1 } else { page_size };
                let total_items = conn
                    .interact(|conn| table.count().get_result(conn))
                    .await??;
                let items = conn
                    .interact(move |conn| {
                        table
                            .limit(page_size)
                            .offset(page * page_size)
                            .load::<super::#name>(conn)
                    })
                    .await??;
                Ok(PaginationResult {
                    items,
                    total_items,
                    page,
                    page_size,
                    num_pages: total_items / page_size + i64::from(total_items % page_size != 0),
                })
            }
            pub async fn delete(conn: &mut Connection, id: i32) -> Result<usize> {
                let result = conn
                    .interact(move |conn| diesel::delete(table.filter(module::id.eq(id))).execute(conn))
                    .await??;
                Ok(result)
            }
        }
    };
    gen.into()
}

