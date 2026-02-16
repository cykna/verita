use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::{Ident, Token, Type, parse_macro_input};

#[proc_macro]
pub fn define_rpc(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DefineRpcInput);

    let mut request_variants = vec![];
    let mut response_variants = vec![];
    let mut client_methods = vec![];
    let mut server_methods = vec![];
    let mut dispatch_arms = vec![];

    for entry in input.entries {
        let method_ident = entry.method_ident; // ex: register
        let req_struct_ident = entry.req_struct_ident; // ex: UserRegistration
        let resp_ty = entry.resp_ty;

        // Variante do enum em UPPER_SNAKE_CASE
        let variant_ident = format_ident!(
            "{}",
            heck::AsShoutySnakeCase(method_ident.to_string()).to_string()
        );

        // Request variant
        let variant = quote! { #variant_ident(#req_struct_ident) };
        request_variants.push(variant);

        // Response variant
        response_variants.push(quote! { #variant_ident(#resp_ty) });

        // Client method: register, login, etc.
        let client_method = quote! {
            async fn #method_ident(
                &mut self,
                request: #req_struct_ident,
            ) -> Result<#resp_ty, Self::Err>;
        };
        client_methods.push(client_method);

        // Server handler: handle_register, handle_login, etc.
        let handle_ident = format_ident!("handle_{}", method_ident);
        let server_method = quote! {
            async fn #handle_ident(
                &mut self,
                request: #req_struct_ident,
            ) -> Result<#resp_ty, Self::Err>;
        };
        server_methods.push(server_method);

        // Dispatch arm
        let dispatch_arm = quote! {
            VeritaRequest::#variant_ident(inner) => {
                match service.#handle_ident(inner).await.map(|r| r.into()) {
                    Ok(v) => VeritaResponse::#variant_ident(v),
                    Err(e) => VeritaResponse::Err(e.into()),
                }
            }
        };
        dispatch_arms.push(dispatch_arm);
    }

    let output = quote! {

        #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, Debug)]
        #[rkyv(derive(Debug))]
        pub enum VeritaRequest {
            #(#request_variants,)*
        }

        #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, Debug)]
        #[rkyv(derive(Debug))]
        pub enum VeritaResponse
        {
            #(#response_variants,)*
            Err(VeritaError),
        }

        #[async_trait::async_trait]
        pub trait VeritaRpcClient: Send + Sync + 'static {
            type Err: Into<VeritaError> + Send + Sync + 'static;
            #(#client_methods)*
        }

        #[async_trait::async_trait]
        pub trait VeritaRpcServer: Send + Sync + 'static {
            type Err: Into<VeritaError> + Send + Sync + 'static;
            #(#server_methods)*
        }

        pub async fn dispatch_verita_request<S, E>(service: &mut S, req: VeritaRequest) -> VeritaResponse
        where
            S: VeritaRpcServer + ?Sized,
       {
            match req {
                #(#dispatch_arms)*

            }
        }
    };

    output.into()
}

// Parser atualizado (sem handle_)
struct DefineRpcInput {
    entries: Vec<RpcEntry>,
}

struct RpcEntry {
    method_ident: Ident,
    req_struct_ident: Ident,
    resp_ty: Type,
}

impl Parse for DefineRpcInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut entries = vec![];

        while !input.is_empty() {
            entries.push(input.parse()?);

            // Consome ; se existir (opcional)
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }
        }

        Ok(Self { entries })
    }
}

impl Parse for RpcEntry {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let method_ident: Ident = input.parse()?;

        let content;
        syn::parenthesized!(content in input);
        let req_struct_ident: Ident = content.parse()?;

        input.parse::<Token![->]>()?;
        let resp_ty: Type = input.parse()?;

        // Consome o `;`
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }

        Ok(Self {
            method_ident,
            req_struct_ident,
            resp_ty,
        })
    }
}
