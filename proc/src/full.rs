use proc_macro2::TokenStream;
use syn::{parse::Parser, spanned::Spanned};

pub fn nat_expr(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut impl_only = false;
    syn::meta::parser(|meta| {
        if meta.path.is_ident("impl_only") {
            impl_only = true;
            Ok(())
        } else {
            Err(meta.error("Unsupported property"))
        }
    })
    .parse2(attr)?;

    let syn::ItemType {
        attrs,
        vis,
        type_token,
        ident,
        generics:
            syn::Generics {
                lt_token,
                mut params,
                gt_token,
                where_clause,
            },
        eq_token,
        ty,
        semi_token,
    } = syn::parse2::<syn::ItemType>(input)?;

    let mut struct_args = proc_macro2::TokenStream::new();
    for param in &params {
        use quote::ToTokens;
        match param {
            syn::GenericParam::Lifetime(param) => param.lifetime.to_tokens(&mut struct_args),
            syn::GenericParam::Type(param) => param.ident.to_tokens(&mut struct_args),
            syn::GenericParam::Const(param) => param.ident.to_tokens(&mut struct_args),
        }
    }

    let expr_impl = quote::quote! {
        impl #lt_token #params #gt_token ::gnat::NatExpr for #ident #lt_token #struct_args #gt_token #where_clause {
            #type_token Eval #eq_token ::gnat::Eval<#ty> #semi_token
        }
    };

    Ok(if impl_only {
        let syn::Visibility::Inherited = vis else {
            return Err(syn::Error::new_spanned(
                vis,
                "Cannot use visibility with `impl_only`",
            ));
        };
        quote::quote! {
            #(#attrs)*
            #expr_impl
        }
    } else {
        let struct_params = params.iter_mut().map(|param| match param {
            syn::GenericParam::Lifetime(param) => syn::GenericParam::Lifetime(syn::LifetimeParam {
                attrs: vec![],
                lifetime: param.lifetime.clone(),
                colon_token: None,
                bounds: Default::default(),
            }),
            syn::GenericParam::Type(param) => syn::GenericParam::Type(syn::TypeParam {
                attrs: vec![],
                ident: param.ident.clone(),
                colon_token: None,
                bounds: Default::default(),
                eq_token: param.eq_token.take(),
                default: param.default.take(),
            }),
            syn::GenericParam::Const(param) => syn::GenericParam::Const(syn::ConstParam {
                attrs: vec![],
                const_token: param.const_token,
                ident: param.ident.clone(),
                colon_token: param.colon_token,
                ty: param.ty.clone(),
                eq_token: param.eq_token.take(),
                default: param.default.take(),
            }),
        });
        let struct_params: Vec<_> = struct_params.collect();

        let struct_fields = params.iter().filter_map(|param| match param {
            syn::GenericParam::Type(syn::TypeParam { ident, .. }) => Some(quote::quote! { #ident }),
            syn::GenericParam::Lifetime(syn::LifetimeParam { lifetime, .. }) => {
                Some(quote::quote! { &#lifetime () })
            }
            _ => None,
        });

        quote::quote! {
            #(#attrs)*
            #vis struct #ident #lt_token #(#struct_params),* #gt_token ( ::core::marker::PhantomData<(#(#struct_fields),*)> );

            #expr_impl
        }
    })
}

fn no_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<()> {
    match &*attrs {
        [] => Ok(()),
        [attr, ..] => Err(syn::Error::new_spanned(attr, "Attrs are unsupported")),
    }
}

pub fn expr(input: TokenStream) -> syn::Result<TokenStream> {
    fn check_nat_lit(lit: &syn::Lit) -> Result<(), &'static str> {
        let syn::Lit::Int(num) = lit else {
            return Err("Non-integer literals can only be used in argument position");
        };
        if num.suffix().is_empty() {
            Ok(())
        } else {
            Err("Suffixed literals can only be used in argument position")
        }
    }
    fn visit_block(expr: syn::Block) -> syn::Result<syn::Type> {
        let mut stmts = expr.stmts.into_iter();
        match [stmts.next(), stmts.next()] {
            [None, _] => Err(syn::Error::new(
                expr.brace_token.span.join(),
                "Missing body",
            )),
            [Some(syn::Stmt::Expr(expr, None)), None] => visit(expr),
            [Some(stmt), None] => Err(syn::Error::new_spanned(
                stmt,
                "Unsupported statement, expected expression",
            )),
            [Some(_), Some(stmt)] => Err(syn::Error::new_spanned(
                stmt,
                "Body must consist of single expression",
            )),
        }
    }
    fn func_path(name: &str, span: proc_macro2::Span) -> Box<syn::Expr> {
        let ident = syn::Ident::new(name, span);
        syn::parse_quote_spanned! { span=> ::gnat::expr::#ident }
    }
    fn visit_arg(expr: syn::Expr) -> syn::Result<syn::GenericArgument> {
        match expr {
            syn::Expr::Const(syn::ExprConst {
                attrs,
                const_token: _,
                block,
            }) => Ok(syn::GenericArgument::Const(
                syn::ExprBlock {
                    attrs,
                    label: None,
                    block,
                }
                .into(),
            )),
            syn::Expr::Lit(ref lit) if check_nat_lit(&lit.lit).is_err() => {
                Ok(syn::GenericArgument::Const(expr))
            }
            _ => visit(expr).map(syn::GenericArgument::Type),
        }
    }
    fn visit(expr: syn::Expr) -> syn::Result<syn::Type> {
        Ok(match expr {
            syn::Expr::Const(expr) => {
                return Err(syn::Error::new_spanned(
                    expr,
                    "Const block may only appear in argument position",
                ));
            }
            syn::Expr::Unary(syn::ExprUnary { attrs, op, expr }) => {
                let span = op.span();
                visit(
                    syn::ExprCall {
                        args: FromIterator::from_iter([*expr]),
                        attrs,
                        paren_token: syn::token::Paren(span),
                        func: match op {
                            syn::UnOp::Not(_) => func_path("IsZero", span),
                            _ => return Err(syn::Error::new_spanned(op, "Unimplemented unary op")),
                        },
                    }
                    .into(),
                )?
            }
            syn::Expr::Binary(syn::ExprBinary {
                attrs,
                left,
                op,
                right,
            }) => {
                let span = op.span();
                visit(
                    syn::ExprCall {
                        args: FromIterator::from_iter([*left, *right]),
                        attrs,
                        paren_token: syn::token::Paren(span),
                        func: func_path(
                            match op {
                                syn::BinOp::Add(_) => "Add",
                                syn::BinOp::Sub(_) => "SatSub",
                                syn::BinOp::Mul(_) => "Mul",
                                syn::BinOp::Div(_) => "Div",
                                syn::BinOp::Rem(_) => "Rem",
                                syn::BinOp::BitXor(_) => "BitXor",
                                syn::BinOp::BitAnd(_) => "BitAnd",
                                syn::BinOp::BitOr(_) => "BitOr",
                                syn::BinOp::Shl(_) => "Shl",
                                syn::BinOp::Shr(_) => "Shr",
                                syn::BinOp::Eq(_) => "Eq",
                                syn::BinOp::Lt(_) => "Lt",
                                syn::BinOp::Le(_) => "Le",
                                syn::BinOp::Ne(_) => "Ne",
                                syn::BinOp::Ge(_) => "Ge",
                                syn::BinOp::Gt(_) => "Gt",
                                //syn::BinOp::And(_) => todo!(),
                                //syn::BinOp::Or(_) => todo!(),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        op,
                                        "Unimplemented binary op",
                                    ));
                                }
                            },
                            span,
                        ),
                    }
                    .into(),
                )?
            }
            syn::Expr::If(expr_if) => {
                let syn::ExprIf {
                    attrs,
                    if_token,
                    cond,
                    then_branch,
                    else_branch: Some((_, else_branch)),
                } = expr_if
                else {
                    return Err(syn::Error::new_spanned(expr_if, "Else branch is required"));
                };

                visit(
                    syn::ExprCall {
                        attrs,
                        func: func_path("If", if_token.span),
                        paren_token: Default::default(),
                        args: FromIterator::from_iter([
                            *cond,
                            syn::ExprBlock {
                                block: then_branch,
                                attrs: vec![],
                                label: None,
                            }
                            .into(),
                            *else_branch,
                        ]),
                    }
                    .into(),
                )?
            }
            syn::Expr::Lit(syn::ExprLit { attrs, lit }) => {
                no_attrs(attrs)?;
                if let Err(err) = check_nat_lit(&lit) {
                    return Err(syn::Error::new_spanned(lit, err));
                };
                syn::parse_quote! { ::gnat::lit!(#lit) }
            }
            syn::Expr::Call(syn::ExprCall {
                attrs,
                func,
                paren_token,
                args,
            }) => {
                no_attrs(attrs)?;
                let syn::Expr::Path(syn::ExprPath {
                    attrs,
                    qself,
                    mut path,
                }) = *func
                else {
                    return Err(syn::Error::new_spanned(func, "Unsupported expr kind"));
                };
                no_attrs(attrs)?;
                let Some(
                    segment @ syn::PathSegment {
                        arguments: syn::PathArguments::None,
                        ..
                    },
                ) = path.segments.last_mut()
                else {
                    return Err(syn::Error::new_spanned(
                        path,
                        "Final path segment of call must not have generic arguments",
                    ));
                };
                segment.arguments =
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: Default::default(),
                        lt_token: syn::Token![<](paren_token.span.open()),
                        args: args
                            .into_iter()
                            .map(visit_arg)
                            .collect::<syn::Result<_>>()?,
                        gt_token: syn::Token![>](paren_token.span.close()),
                    });

                syn::Type::Path(syn::TypePath { qself, path })
            }
            syn::Expr::Infer(syn::ExprInfer {
                attrs,
                underscore_token,
            }) => {
                no_attrs(attrs)?;
                syn::Type::Infer(syn::TypeInfer { underscore_token })
            }
            syn::Expr::Macro(syn::ExprMacro { attrs, mac }) => {
                no_attrs(attrs)?;
                syn::Type::Macro(syn::TypeMacro { mac })
            }
            syn::Expr::Path(syn::ExprPath { attrs, qself, path }) => {
                no_attrs(attrs)?;
                syn::Type::Path(syn::TypePath { qself, path })
            }
            syn::Expr::Group(syn::ExprGroup {
                attrs,
                group_token: _,
                expr,
            }) => {
                no_attrs(attrs)?;
                visit(*expr)?
            }
            syn::Expr::Paren(syn::ExprParen {
                attrs,
                paren_token: _,
                expr,
            }) => {
                no_attrs(attrs)?;
                visit(*expr)?
            }
            syn::Expr::Block(syn::ExprBlock {
                attrs,
                label: _,
                block,
            }) => {
                no_attrs(attrs)?;
                visit_block(block)?
            }
            // TODO: Allow guard clauses
            // syn::Expr::Return(expr_return) => todo!(),
            _ => return Err(syn::Error::new_spanned(expr, "Unsupported expr kind")),
        })
    }

    visit(syn::parse2(input)?).map(quote::ToTokens::into_token_stream)
}
