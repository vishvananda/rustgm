    // The number of fields that can be handled without an array.
    const CUTOFF: usize = 5;

    fn expr_for_field(
        cx: &ExtCtxt<'_>,
        field: &FieldInfo,
        index: usize,
        len: usize,
    ) -> Box<ast::Expr> {
        if index < len - 1 {
            field.self_expr.clone()
        } else {
            // Unsized types need an extra indirection, but only the last field
            // may be unsized.
            cx.expr_addr_of(field.span, field.self_expr.clone())
        }
    }

    if fields.is_empty() {
        // Special case for no fields.
        let fn_path_write_str = cx.std_path(&[sym::fmt, sym::Formatter, sym::write_str]);
        let expr = cx.expr_call_global(span, fn_path_write_str, thin_vec![fmt, name]);
        BlockOrExpr::new_expr(expr)
    } else if fields.len() <= CUTOFF {
        // Few enough fields that we can use a specific-length method.
        let debug = if is_struct {
            format!("debug_struct_field{}_finish", fields.len())
        } else {
            format!("debug_tuple_field{}_finish", fields.len())
        };
        let fn_path_debug = cx.std_path(&[sym::fmt, sym::Formatter, Symbol::intern(&debug)]);

        let mut args = ThinVec::with_capacity(2 + fields.len() * args_per_field);
        args.extend([fmt, name]);
        for i in 0..fields.len() {
            let field = &fields[i];
            if is_struct {
                let name = cx.expr_str(field.span, field.name.unwrap().name);
                args.push(name);
            }

            let field = expr_for_field(cx, field, i, fields.len());
            args.push(field);
        }
        let expr = cx.expr_call_global(span, fn_path_debug, args);
        BlockOrExpr::new_expr(expr)
    } else {
        // Enough fields that we must use the any-length method.
        let mut name_exprs = ThinVec::with_capacity(fields.len());
        let mut value_exprs = ThinVec::with_capacity(fields.len());

        for i in 0..fields.len() {
            let field = &fields[i];
            if is_struct {
                name_exprs.push(cx.expr_str(field.span, field.name.unwrap().name));
            }

            let field = expr_for_field(cx, field, i, fields.len());
            value_exprs.push(field);
        }

        // `let names: &'static _ = &["field1", "field2"];`
        let names_let = is_struct.then(|| {
            let lt_static = Some(cx.lifetime_static(span));
            let ty_static_ref = cx.ty_ref(span, cx.ty_infer(span), lt_static, ast::Mutability::Not);
            cx.stmt_let_ty(
                span,
                false,
                Ident::new(sym::names, span),
                Some(ty_static_ref),
                cx.expr_array_ref(span, name_exprs),
            )
        });

        // `let values: &[&dyn Debug] = &[&&self.field1, &&self.field2];`
        let path_debug = cx.path_global(span, cx.std_path(&[sym::fmt, sym::Debug]));
        let ty_dyn_debug = cx.ty(
            span,
            ast::TyKind::TraitObject(
                vec![cx.trait_bound(path_debug, false)],
                ast::TraitObjectSyntax::Dyn,
            ),
        );
        let ty_slice = cx.ty(
            span,
            ast::TyKind::Slice(cx.ty_ref(span, ty_dyn_debug, None, ast::Mutability::Not)),
        );
        let values_let = cx.stmt_let_ty(
            span,
            false,
            Ident::new(sym::values, span),
            Some(cx.ty_ref(span, ty_slice, None, ast::Mutability::Not)),
            cx.expr_array_ref(span, value_exprs),
        );

        // `fmt::Formatter::debug_struct_fields_finish(fmt, name, names, values)` or
        // `fmt::Formatter::debug_tuple_fields_finish(fmt, name, values)`
        let sym_debug = if is_struct {
            sym::debug_struct_fields_finish
        } else {
            sym::debug_tuple_fields_finish
        };
        let fn_path_debug_internal = cx.std_path(&[sym::fmt, sym::Formatter, sym_debug]);

        let mut args = ThinVec::with_capacity(4);
        args.push(fmt);
        args.push(name);
        if is_struct {
            args.push(cx.expr_ident(span, Ident::new(sym::names, span)));
        }
        args.push(cx.expr_ident(span, Ident::new(sym::values, span)));
        let expr = cx.expr_call_global(span, fn_path_debug_internal, args);

        let mut stmts = ThinVec::with_capacity(2);
        if is_struct {
            stmts.push(names_let.unwrap());
        }
        stmts.push(values_let);
        BlockOrExpr::new_mixed(stmts, Some(expr))
    }
}
