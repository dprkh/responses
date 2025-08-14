use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, FnArg, ItemFn, Lit, LitStr, Meta, 
    Pat, PatType, Type
};

/// Main procedural macro that transforms functions into tools
#[proc_macro_attribute]
pub fn tool(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = if args.is_empty() {
        None
    } else {
        Some(parse_macro_input!(args as LitStr))
    };
    let input_fn = parse_macro_input!(input as ItemFn);
    
    match generate_tool_function(args, input_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_tool_function(args: Option<LitStr>, input_fn: ItemFn) -> syn::Result<TokenStream2> {
    let fn_name = &input_fn.sig.ident;
    
    // Extract tool name from arguments or use function name
    let tool_name = args.map(|lit| lit.value()).unwrap_or_else(|| fn_name.to_string());
    
    // Extract function documentation
    let fn_doc = extract_doc_comment(&input_fn.attrs);
    
    // Parse function parameters (excluding context parameters) 
    let (params, context_param) = parse_function_parameters(&input_fn.sig.inputs)?;
    
    // Generate parameter struct name
    let params_struct_name = format_ident!("{}Params", capitalize_first_letter(&fn_name.to_string()));
    
    // Generate the parameter struct
    let params_struct = generate_params_struct(&params_struct_name, &params)?;
    
    // Generate the function handler
    let handler_struct_name = format_ident!("{}Handler", capitalize_first_letter(&fn_name.to_string()));
    let handler_impl = generate_handler_impl(&handler_struct_name, &params_struct_name, &tool_name, &fn_doc, &input_fn, &params, &context_param)?;
    
    // Generate the wrapper function that calls the original
    let wrapper_fn = generate_wrapper_function(&input_fn, &params, &context_param)?;
    
    // Generate the handler creator function
    let handler_creator = generate_handler_creator(fn_name, &handler_struct_name)?;
    
    Ok(quote! {
        #params_struct
        #handler_impl
        #wrapper_fn
        #handler_creator
    })
}


fn extract_doc_comment(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc")
                && let Meta::NameValue(meta) = &attr.meta
                && let syn::Expr::Lit(lit) = &meta.value
                && let Lit::Str(lit_str) = &lit.lit
            {
                return Some(lit_str.value().trim().to_string());
            }
            None
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone)]
struct Parameter {
    name: Ident,
    ty: Type,
    doc: String,
}

fn parse_function_parameters(inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>) -> syn::Result<(Vec<Parameter>, Option<Ident>)> {
    let mut params = Vec::new();
    let mut context_param = None;
    
    for input in inputs {
        match input {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                if let Pat::Ident(pat_ident) = pat.as_ref() {
                    let param_name = &pat_ident.ident;
                    
                    // Check if this is a context parameter (by name or reference type)
                    if param_name == "context" || is_reference_type(ty) {
                        context_param = Some(param_name.clone());
                        continue;
                    }
                    
                    // For now, no parameter documentation since it's not stable
                    params.push(Parameter {
                        name: param_name.clone(),
                        ty: (**ty).clone(),
                        doc: String::new(),
                    });
                }
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(input, "Self parameter not supported"));
            }
        }
    }
    
    Ok((params, context_param))
}

fn is_reference_type(ty: &Type) -> bool {
    matches!(ty, Type::Reference(_))
}

fn generate_params_struct(struct_name: &Ident, params: &[Parameter]) -> syn::Result<TokenStream2> {
    let fields = params.iter().map(|param| {
        let name = &param.name;
        let ty = &param.ty;
        let doc = &param.doc;
        
        if doc.is_empty() {
            quote! { pub #name: #ty }
        } else {
            quote! {
                #[doc = #doc]
                pub #name: #ty
            }
        }
    });
    
    Ok(quote! {
        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
        pub struct #struct_name {
            #(#fields,)*
        }
    })
}

fn generate_handler_impl(
    handler_name: &Ident, 
    params_name: &Ident, 
    tool_name: &str, 
    description: &str,
    input_fn: &ItemFn,
    params: &[Parameter],
    context_param: &Option<Ident>,
) -> syn::Result<TokenStream2> {
    let invoke_method = generate_invoke_method(input_fn, params_name, params, context_param)?;
    let schema_properties = generate_schema_properties(params)?;
    
    Ok(quote! {
        #[derive(Clone)]
        pub struct #handler_name;
        
        impl responses::functions::FunctionHandler for #handler_name {
            type Parameters = #params_name;
            
            fn name(&self) -> &str {
                #tool_name
            }
            
            fn tool(&self) -> responses::types::Tool {
                use responses::types::{Tool, ToolFunction};
                use schemars::Schema;
                use serde_json::{json, Value};
                
                let schema = Self::build_schema();
                Tool::Function(ToolFunction {
                    name: #tool_name.to_string(),
                    description: if #description.is_empty() { None } else { Some(#description.to_string()) },
                    parameters: schema,
                    strict: None,
                })
            }
        }
        
        impl #handler_name {
            fn build_schema() -> schemars::Schema {
                use serde_json::{json, Value, Map};
                
                #schema_properties
                
                let schema_value = json!({
                    "type": "object",
                    "properties": properties,
                    "required": required_fields,
                    "additionalProperties": false
                });
                
                serde_json::from_value(schema_value).unwrap()
            }
            
            #invoke_method
        }
    })
}

fn generate_wrapper_function(
    original_fn: &ItemFn,
    _params: &[Parameter],
    _context_param: &Option<Ident>,
) -> syn::Result<TokenStream2> {
    // Just return the original function unchanged
    Ok(quote! { #original_fn })
}

fn generate_handler_creator(fn_name: &Ident, handler_name: &Ident) -> syn::Result<TokenStream2> {
    let creator_name = format_ident!("{}_handler", fn_name);
    
    Ok(quote! {
        pub fn #creator_name() -> #handler_name {
            #handler_name
        }
    })
}

fn generate_invoke_method(
    input_fn: &ItemFn,
    params_name: &Ident,
    params: &[Parameter],
    context_param: &Option<Ident>,
) -> syn::Result<TokenStream2> {
    let fn_name = &input_fn.sig.ident;
    let is_async = input_fn.sig.asyncness.is_some();
    let return_type = extract_inner_return_type(&input_fn.sig.output)?;
    
    // Generate parameter extraction from the struct
    let param_extractions = params.iter().map(|param| {
        let name = &param.name;
        quote! { params.#name }
    });
    
    // Generate the function call
    let function_call = if let Some(_context) = context_param {
        // Function has context parameter
        if is_async {
            quote! {
                #fn_name(#(#param_extractions,)* context).await
            }
        } else {
            quote! {
                #fn_name(#(#param_extractions,)* context)
            }
        }
    } else {
        // Function has no context parameter
        if is_async {
            quote! {
                #fn_name(#(#param_extractions,)*).await
            }
        } else {
            quote! {
                #fn_name(#(#param_extractions,)*)
            }
        }
    };
    
    // Generate the invoke method with automatic name validation
    let invoke_method = if let Some(context) = context_param {
        let context_type = get_context_type(input_fn, context)?;
        if is_async {
            quote! {
                pub async fn invoke(
                    &self, 
                    call: &responses::types::OutputFunctionCall, 
                    context: &#context_type
                ) -> responses::error::Result<Option<#return_type>> {
                    if call.name != self.name() {
                        return Ok(None);
                    }
                    
                    let params: #params_name = serde_json::from_str(&call.arguments)
                        .map_err(|e| responses::error::Error::FunctionParameterParsing { 
                            function_name: call.name.clone(), 
                            source: e 
                        })?;
                    let result = #function_call?;
                    Ok(Some(result))
                }
            }
        } else {
            quote! {
                pub fn invoke(
                    &self, 
                    call: &responses::types::OutputFunctionCall, 
                    context: &#context_type
                ) -> responses::error::Result<Option<#return_type>> {
                    if call.name != self.name() {
                        return Ok(None);
                    }
                    
                    let params: #params_name = serde_json::from_str(&call.arguments)
                        .map_err(|e| responses::error::Error::FunctionParameterParsing { 
                            function_name: call.name.clone(), 
                            source: e 
                        })?;
                    let result = #function_call?;
                    Ok(Some(result))
                }
            }
        }
    } else {
        // No context parameter
        if is_async {
            quote! {
                pub async fn invoke(
                    &self, 
                    call: &responses::types::OutputFunctionCall
                ) -> responses::error::Result<Option<#return_type>> {
                    if call.name != self.name() {
                        return Ok(None);
                    }
                    
                    let params: #params_name = serde_json::from_str(&call.arguments)
                        .map_err(|e| responses::error::Error::FunctionParameterParsing { 
                            function_name: call.name.clone(), 
                            source: e 
                        })?;
                    let result = #function_call?;
                    Ok(Some(result))
                }
            }
        } else {
            quote! {
                pub fn invoke(
                    &self, 
                    call: &responses::types::OutputFunctionCall
                ) -> responses::error::Result<Option<#return_type>> {
                    if call.name != self.name() {
                        return Ok(None);
                    }
                    
                    let params: #params_name = serde_json::from_str(&call.arguments)
                        .map_err(|e| responses::error::Error::FunctionParameterParsing { 
                            function_name: call.name.clone(), 
                            source: e 
                        })?;
                    let result = #function_call?;
                    Ok(Some(result))
                }
            }
        }
    };
    
    Ok(invoke_method)
}

fn extract_inner_return_type(output: &syn::ReturnType) -> syn::Result<Type> {
    match output {
        syn::ReturnType::Default => {
            // Function returns ()
            Ok(syn::parse_quote!(()))
        }
        syn::ReturnType::Type(_, ty) => {
            // Check if it's Result<T, E> and extract T
            if let Type::Path(type_path) = ty.as_ref()
                && let Some(segment) = type_path.path.segments.last()
                && segment.ident == "Result"
                && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
            {
                return Ok(inner_type.clone());
            }
            // If not Result<T, E>, return the type as-is
            Ok((**ty).clone())
        }
    }
}

fn get_context_type(input_fn: &ItemFn, context_param: &Ident) -> syn::Result<Type> {
    // Find the context parameter in the function signature and extract its type
    for input in &input_fn.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = input
            && let Pat::Ident(pat_ident) = pat.as_ref()
            && pat_ident.ident == *context_param
        {
            // Extract the type from &SomeType to SomeType
            if let Type::Reference(type_ref) = ty.as_ref() {
                return Ok(*type_ref.elem.clone());
            }
            return Ok((**ty).clone());
        }
    }
    Err(syn::Error::new_spanned(context_param, "Context parameter not found"))
}


fn generate_schema_properties(params: &[Parameter]) -> syn::Result<TokenStream2> {
    let mut property_entries = Vec::new();
    let mut required_fields = Vec::new();
    
    for param in params {
        let name = param.name.to_string();
        let field_schema = generate_field_schema(&param.ty)?;
        
        property_entries.push(quote! {
            properties.insert(#name.to_string(), #field_schema);
        });
        
        // Check if the field is required (not Option<T>)
        if !is_option_type(&param.ty) {
            required_fields.push(quote! {
                required_fields.push(#name.to_string());
            });
        }
    }
    
    Ok(quote! {
        let mut properties = serde_json::Map::new();
        let mut required_fields: Vec<String> = Vec::new();
        
        #(#property_entries)*
        #(#required_fields)*
    })
}

fn generate_field_schema(ty: &Type) -> syn::Result<TokenStream2> {
    // For now, we'll handle basic types. This can be extended later.
    if is_option_type(ty) {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        // For Option<String>, Option<f64>, etc.
                        return generate_inner_field_schema(inner_type);
                    }
                }
            }
        }
    }
    
    generate_inner_field_schema(ty)
}

fn generate_inner_field_schema(ty: &Type) -> syn::Result<TokenStream2> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();
            return Ok(match type_name.as_str() {
                "String" | "str" => quote! { json!({"type": "string", "description": "A text value"}) },
                "f64" | "f32" => quote! { json!({"type": "number", "format": "double", "description": "A floating point number"}) },
                "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => quote! { json!({"type": "integer", "description": "An integer number"}) },
                "i8" | "i16" | "u8" | "u16" => quote! { json!({"type": "integer", "description": "An integer number"}) },
                "bool" => quote! { json!({"type": "boolean", "description": "A boolean value"}) },
                "Vec" => {
                    // Handle Vec<T> types
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_schema = generate_inner_field_schema(inner_type)?;
                            return Ok(quote! { 
                                json!({
                                    "type": "array", 
                                    "items": #inner_schema,
                                    "description": "An array of values"
                                }) 
                            });
                        }
                    }
                    quote! { json!({"type": "array", "items": {"type": "string"}, "description": "An array of values"}) }
                },
                "HashMap" | "BTreeMap" => quote! { 
                    json!({
                        "type": "object", 
                        "additionalProperties": true,
                        "description": "A map of key-value pairs"
                    }) 
                },
                _ => quote! { json!({"type": "string", "description": "A text value"}) }, // Default to string
            });
        }
    }
    
    Ok(quote! { json!({"type": "string", "description": "A text value"}) }) // Default fallback
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn capitalize_first_letter(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

