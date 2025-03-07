use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Field, Fields, GenericParam, Lit, Meta, Path,
    PathArguments, TypeParam,
};

#[proc_macro_derive(Block, attributes(port, work, qsdr_crate))]
pub fn block_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    //dbg!(&ast);
    let qsdr = qsdr_crate(&ast);
    let vis = &ast.vis;

    let work = work_type(&ast);

    let block_ident = &ast.ident;
    let block_generics = struct_generic_types(&ast);
    let block_where = &ast.generics.where_clause;

    let Data::Struct(data) = &ast.data else {
        panic!("derive(Block) only works for struct");
    };
    let Fields::Named(fields) = &data.fields else {
        panic!("struct fields should be be named fields");
    };
    let ports = fields
        .named
        .iter()
        .filter(|field| field_is_port(field))
        .collect::<Vec<_>>();

    let work_impl = match work {
        WorkType::WorkInPlace => {
            check_required_ports(&ports, &["input", "output"], "WorkInPlace");
            quote! {
                async fn block_work(&mut self, channels: &mut Self::Channels) -> Result<#qsdr::BlockWorkStatus> {
                    use #qsdr::{Receiver, Sender};
                    let Some(mut item) = channels.input.recv().await else {
                        return Ok(#qsdr::BlockWorkStatus::Done);
                    };
                    let status = self.work_in_place(&mut item).await?;
                    if status.produces_output() {
                        channels.output.send(item);
                    }
                    Ok(status.into())
                }
            }
        }
        WorkType::WorkSink => {
            check_required_ports(&ports, &["input"], "WorkSink");
            quote! {
                async fn block_work(&mut self, channels: &mut Self::Channels) -> Result<#qsdr::BlockWorkStatus> {
                    use #qsdr::{Receiver, RefReceiver, Sender};
                    use ::std::borrow::Borrow;
                    let Some(item) = channels.input.ref_recv().await else {
                         return Ok(#qsdr::BlockWorkStatus::Done);
                    };
                    self.work_sink(item.borrow()).await
                }
            }
        }
        WorkType::WorkWithRef => {
            check_required_ports(&ports, &["input", "source", "output"], "WorkWithRef");
            quote! {
                async fn block_work(&mut self, channels: &mut Self::Channels) -> Result<#qsdr::BlockWorkStatus> {
                    use #qsdr::{Receiver, RefReceiver, Sender};
                    use ::std::borrow::Borrow;
                    let Some(mut output_item) = channels.source.recv().await else {
                        return Ok(#qsdr::BlockWorkStatus::Done);
                    };
                    let Some(input_item) = channels.input.ref_recv().await else {
                         return Ok(#qsdr::BlockWorkStatus::Done);
                    };
                    let status = self.work_with_ref(input_item.borrow(), &mut output_item).await?;
                    // drop the input item reference, which potentially causes it to be returned
                    drop(input_item);
                    if status.produces_output() {
                        channels.output.send(output_item);
                    }
                    Ok(status.into())
                }
            }
        }
        WorkType::WorkCustom => {
            quote! {
                fn block_work(&mut self, channels: &mut Self::Channels)
                              -> impl ::std::future::Future<Output = Result<#qsdr::BlockWorkStatus>> {
                    #qsdr::WorkCustom::work_custom(self, channels)
                }
            }
        }
    };

    let mut channels = Vec::new();
    let mut channel_idents = Vec::new();
    let mut seeds = Vec::new();
    let mut seeds_defaults = Vec::new();
    let mut port_ids = Vec::new();
    for (port_id, port) in ports.iter().enumerate() {
        let ident = port.ident.as_ref().expect("port should have ident");
        channel_idents.push(ident);
        let ty = &port.ty;
        channels.push(quote! {
            #ident: <#ty as #qsdr::__private::Port>::Channel
        });
        seeds.push(quote! {
            #ident: ::std::cell::RefCell<<#ty as #qsdr::__private::Port>::Seed>
        });
        seeds_defaults.push(quote! {
            #ident: ::std::cell::RefCell::new(Default::default())
        });
        let port_id = u32::try_from(port_id).unwrap();
        port_ids.push(quote! {
            #vis fn #ident(&self) -> #qsdr::ports::Endpoint<'_, #ty> {
                // Use this to remove a "field is never read" warning. With
                // this, the warning will typically show iff this function is
                // never called.
                let _ = &self.as_ref().#ident;
                let port = #qsdr::__private::PortId::from(#port_id);
                let seed = self.seeds.#ident.borrow_mut();
                #qsdr::ports::Endpoint::new(self.flowgraph_id, self.node_id, port, seed)
            }
        });
    }

    let block_channels_ident = format_ident!("__{block_ident}BlockChannels");
    let block_seeds_ident = format_ident!("__{block_ident}BlockSeeds");
    let block_generic_types = block_generics.iter().map(|ty| &ty.ident);
    let block_generic_types = quote! {
        #(#block_generic_types),*
    };
    let block_generic_list = quote! {
        #(#block_generics),*
    };

    let block_channels = quote! {
        #qsdr::__private::pin_project_lite::pin_project! {
            #vis struct #block_channels_ident<#block_generic_list>
            #block_where
        {
            #(
                #[pin]
                #channels
            ),*,
            __qsdr__phantom: ::std::marker::PhantomData<(#block_generic_types)>,
        }
        }

        impl<#block_generic_list> TryFrom<#block_seeds_ident<#block_generic_types>>
            for #block_channels_ident<#block_generic_types>
            #block_where
        {
            type Error = anyhow::Error;

            fn try_from(value: #block_seeds_ident<#block_generic_types>) -> anyhow::Result<Self> {
                Ok(Self {
                    #(#channel_idents: value.#channel_idents.into_inner().try_into()?),*,
                    __qsdr__phantom: ::std::marker::PhantomData,
                })
            }
        }

        impl<#block_generic_list> ::std::fmt::Debug for #block_channels_ident<#block_generic_types>
            #block_where
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                f.debug_struct("BlockChannels")
                    #(
                        .field(stringify!(#channel_idents), &std::any::type_name_of_val(&self.#channel_idents))
                    )*
                    .field("__qsdr__phantom", &self.__qsdr__phantom)
                    .finish()
            }
        }
    };

    let block_seeds = quote! {
        #vis struct #block_seeds_ident<#block_generic_list>
            #block_where
        {
            #(#seeds),*,
            __qsdr__phantom: ::std::marker::PhantomData<(#block_generic_types)>,
        }

        impl<#block_generic_list> Default for #block_seeds_ident<#block_generic_types>
            #block_where
        {
            fn default() -> Self {
                Self {
                    #(#channel_idents: Default::default()),*,
                    __qsdr__phantom: Default::default(),
                }
            }
        }

        impl<#block_generic_list> ::std::fmt::Debug for #block_seeds_ident<#block_generic_types>
            #block_where
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                f.debug_struct("BlockSeeds")
                    #(
                        .field(stringify!(#channel_idents), &std::any::type_name_of_val(&self.#channel_idents))
                    )*
                    .field("__qsdr__phantom", &self.__qsdr__phantom)
                    .finish()
            }
        }
    };

    let flowgraph_node_ident = format_ident!("__{block_ident}FlowgraphNode");
    let flowgraph_node = quote! {
        #[derive(Debug)]
        #vis struct #flowgraph_node_ident<#block_generic_list>
            #block_where
        {
            flowgraph_id: #qsdr::__private::FlowgraphId,
            node_id: #qsdr::__private::NodeId,
            block: #block_ident<#block_generic_types>,
            seeds: #block_seeds_ident<#block_generic_types>,
        }

        impl<#block_generic_list> #qsdr::__private::FlowgraphNode for #flowgraph_node_ident<#block_generic_types>
            #block_where
        {
            type B = #block_ident<#block_generic_types>;

            fn flowgraph_id(&self) -> #qsdr::__private::FlowgraphId {
                self.flowgraph_id
            }

            fn node_id(&self) -> #qsdr::__private::NodeId {
                self.node_id
            }

            fn wrap_block(flowgraph_id: #qsdr::__private::FlowgraphId,
                          node_id: #qsdr::__private::NodeId, block: Self::B) -> Self {
                Self { flowgraph_id, node_id, block, seeds: Default::default() }
            }

            fn try_into_object(self, _fg: &mut #qsdr::ValidatedFlowgraph) ->
                Result<#qsdr::BlockObject<#block_ident<#block_generic_types>>, anyhow::Error> {
                    Ok(#qsdr::BlockObject::new(self.block, self.seeds.try_into()?))
                }
        }

        impl<#block_generic_list> ::std::convert::AsRef<#block_ident<#block_generic_types>>
            for #flowgraph_node_ident<#block_generic_types>
            #block_where
        {
            fn as_ref(&self) -> &#block_ident<#block_generic_types> {
                &self.block
            }
        }

        impl<#block_generic_list> ::std::convert::AsMut<#block_ident<#block_generic_types>>
            for #flowgraph_node_ident<#block_generic_types>
            #block_where
        {
            fn as_mut(&mut self) -> &mut #block_ident<#block_generic_types> {
                &mut self.block
            }
        }
    };

    let block_impl = quote! {
        impl<#block_generic_list> #qsdr::Block for #block_ident<#block_generic_types>
            #block_where
        {
            type Channels = #block_channels_ident<#block_generic_types>;

            type Seeds = #block_seeds_ident<#block_generic_types>;

            type Node = #flowgraph_node_ident<#block_generic_types>;

            #work_impl
        }
    };

    let ports_impl = quote! {
        impl<#block_generic_list> #flowgraph_node_ident<#block_generic_types>
            #block_where
        {
            #(#port_ids)*
        }
    };

    let code = quote! {
        const _: () =  {
            #block_channels
            #block_seeds
            #flowgraph_node
            #block_impl
            #ports_impl
        };
    };
    //println!("{}", pretty_print(&code));
    code.into()
}

// https://stackoverflow.com/a/74360109
#[allow(dead_code)]
fn pretty_print(ts: &proc_macro2::TokenStream) -> String {
    let file = syn::parse_file(&ts.to_string()).unwrap();
    prettyplease::unparse(&file)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum WorkType {
    WorkInPlace,
    WorkSink,
    WorkWithRef,
    WorkCustom,
}

impl FromStr for WorkType {
    type Err = String;
    fn from_str(s: &str) -> Result<WorkType, String> {
        Ok(match s {
            "WorkInPlace" => WorkType::WorkInPlace,
            "WorkSink" => WorkType::WorkSink,
            "WorkWithRef" => WorkType::WorkWithRef,
            "WorkCustom" => WorkType::WorkCustom,
            _ => return Err(format!("invalid work type: {s}")),
        })
    }
}

fn qsdr_crate(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let qsdr_crate_attrs = ast
        .attrs
        .iter()
        .filter_map(|attr| {
            let Meta::NameValue(name_value) = &attr.meta else {
                return None;
            };
            let segments = &name_value.path.segments;
            if segments.len() != 1 {
                return None;
            }
            let segment = segments.first().unwrap();
            if segment.ident == "qsdr_crate" && matches!(segment.arguments, PathArguments::None) {
                let Expr::Lit(lit) = &name_value.value else {
                    panic!("qsdr_crate value is not a literal");
                };
                let Lit::Str(s) = &lit.lit else {
                    panic!("qsdr_crate value is not a string literal");
                };
                Some(s.parse().unwrap())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if qsdr_crate_attrs.is_empty() {
        return "::qsdr".parse().unwrap();
    }
    if qsdr_crate_attrs.len() > 1 {
        panic!("qsdr_crate attribute present multiple times");
    }
    qsdr_crate_attrs.into_iter().next().unwrap()
}

fn work_type(ast: &DeriveInput) -> WorkType {
    let work_attrs = ast
        .attrs
        .iter()
        .filter_map(|attr| {
            let Meta::List(list) = &attr.meta else {
                return None;
            };
            let segments = &list.path.segments;
            if segments.len() != 1 {
                return None;
            }
            let segment = segments.first().unwrap();
            if segment.ident == "work" && matches!(segment.arguments, PathArguments::None) {
                Some(&list.tokens)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if work_attrs.is_empty() {
        panic!("work attribute missing");
    }
    if work_attrs.len() > 1 {
        panic!("work attribute present multiple times");
    }
    let attr = work_attrs[0].clone().into_iter().collect::<Vec<_>>();
    if attr.len() != 1 {
        panic!("work attribute does not have a single argument");
    }
    let proc_macro2::TokenTree::Ident(ident) = &attr[0] else {
        panic!("work attribute is not an ident");
    };
    match ident.to_string().parse() {
        Ok(w) => w,
        Err(err) => panic!("{}", err),
    }
}

fn struct_generic_types(ast: &DeriveInput) -> Vec<TypeParam> {
    ast.generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(ty) = param {
                let mut ty = ty.clone();
                // remove any possible default, since it interferes with code
                // generation
                ty.default = None;
                Some(ty)
            } else {
                None
            }
        })
        .collect()
}

fn field_is_port(field: &Field) -> bool {
    field.attrs.iter().any(|attr| match &attr.meta {
        Meta::Path(Path { segments, .. }) => {
            if segments.len() != 1 {
                return false;
            }
            let segment = segments.first().unwrap();
            segment.ident == "port" && matches!(segment.arguments, PathArguments::None)
        }
        _ => false,
    })
}

fn has_port_with_name(ports: &[&Field], name: &str) -> bool {
    ports.iter().any(|field| {
        if let Some(ident) = &field.ident {
            ident == name
        } else {
            false
        }
    })
}

fn check_required_ports(ports: &[&Field], required: &[&str], work_name: &str) {
    for req in required {
        if !has_port_with_name(ports, req) {
            panic!("{} requires a port called {}", work_name, req);
        }
    }
}
