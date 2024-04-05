use syn::{visit_mut::VisitMut, Item};

/// Remove #[doc = "..."] from item.
pub fn remove_doc_attrs(mut item: Item) -> Item {
    RemoveDocAttrs::new().visit_item_mut(&mut item);
    item
}

macro_rules! impl_visit_mut {
    ($name:ident, $type:ident) => {
        fn $name(&mut self, i: &mut ::syn::$type) {
            i.attrs.retain(|attr| !attr.path().is_ident("doc"));
            ::syn::visit_mut::$name(self, i);
        }
    };
}

struct RemoveDocAttrs;

impl RemoveDocAttrs {
    fn new() -> Self {
        Self
    }
}

// TODO: I just copy & paste methods that the Type has attrs.
//       Is there any good solution to generate this mechanically?
impl VisitMut for RemoveDocAttrs {
    impl_visit_mut!(visit_arm_mut, Arm);
    impl_visit_mut!(visit_bare_fn_arg_mut, BareFnArg);
    impl_visit_mut!(visit_bare_variadic_mut, BareVariadic);
    impl_visit_mut!(visit_const_param_mut, ConstParam);
    impl_visit_mut!(visit_derive_input_mut, DeriveInput);
    impl_visit_mut!(visit_expr_array_mut, ExprArray);
    impl_visit_mut!(visit_expr_assign_mut, ExprAssign);
    impl_visit_mut!(visit_expr_async_mut, ExprAsync);
    impl_visit_mut!(visit_expr_await_mut, ExprAwait);
    impl_visit_mut!(visit_expr_binary_mut, ExprBinary);
    impl_visit_mut!(visit_expr_block_mut, ExprBlock);
    impl_visit_mut!(visit_expr_break_mut, ExprBreak);
    impl_visit_mut!(visit_expr_call_mut, ExprCall);
    impl_visit_mut!(visit_expr_cast_mut, ExprCast);
    impl_visit_mut!(visit_expr_closure_mut, ExprClosure);
    impl_visit_mut!(visit_expr_const_mut, ExprConst);
    impl_visit_mut!(visit_expr_continue_mut, ExprContinue);
    impl_visit_mut!(visit_expr_field_mut, ExprField);
    impl_visit_mut!(visit_expr_for_loop_mut, ExprForLoop);
    impl_visit_mut!(visit_expr_group_mut, ExprGroup);
    impl_visit_mut!(visit_expr_if_mut, ExprIf);
    impl_visit_mut!(visit_expr_index_mut, ExprIndex);
    impl_visit_mut!(visit_expr_infer_mut, ExprInfer);
    impl_visit_mut!(visit_expr_let_mut, ExprLet);
    impl_visit_mut!(visit_expr_lit_mut, ExprLit);
    impl_visit_mut!(visit_expr_loop_mut, ExprLoop);
    impl_visit_mut!(visit_expr_macro_mut, ExprMacro);
    impl_visit_mut!(visit_expr_match_mut, ExprMatch);
    impl_visit_mut!(visit_expr_method_call_mut, ExprMethodCall);
    impl_visit_mut!(visit_expr_paren_mut, ExprParen);
    impl_visit_mut!(visit_expr_path_mut, ExprPath);
    impl_visit_mut!(visit_expr_range_mut, ExprRange);
    impl_visit_mut!(visit_expr_reference_mut, ExprReference);
    impl_visit_mut!(visit_expr_repeat_mut, ExprRepeat);
    impl_visit_mut!(visit_expr_return_mut, ExprReturn);
    impl_visit_mut!(visit_expr_struct_mut, ExprStruct);
    impl_visit_mut!(visit_expr_try_mut, ExprTry);
    impl_visit_mut!(visit_expr_try_block_mut, ExprTryBlock);
    impl_visit_mut!(visit_expr_tuple_mut, ExprTuple);
    impl_visit_mut!(visit_expr_unary_mut, ExprUnary);
    impl_visit_mut!(visit_expr_unsafe_mut, ExprUnsafe);
    impl_visit_mut!(visit_expr_while_mut, ExprWhile);
    impl_visit_mut!(visit_expr_yield_mut, ExprYield);
    impl_visit_mut!(visit_field_mut, Field);
    impl_visit_mut!(visit_field_pat_mut, FieldPat);
    impl_visit_mut!(visit_field_value_mut, FieldValue);
    impl_visit_mut!(visit_file_mut, File);
    impl_visit_mut!(visit_foreign_item_fn_mut, ForeignItemFn);
    impl_visit_mut!(visit_foreign_item_macro_mut, ForeignItemMacro);
    impl_visit_mut!(visit_foreign_item_static_mut, ForeignItemStatic);
    impl_visit_mut!(visit_foreign_item_type_mut, ForeignItemType);
    impl_visit_mut!(visit_impl_item_const_mut, ImplItemConst);
    impl_visit_mut!(visit_impl_item_fn_mut, ImplItemFn);
    impl_visit_mut!(visit_impl_item_macro_mut, ImplItemMacro);
    impl_visit_mut!(visit_impl_item_type_mut, ImplItemType);
    impl_visit_mut!(visit_item_const_mut, ItemConst);
    impl_visit_mut!(visit_item_enum_mut, ItemEnum);
    impl_visit_mut!(visit_item_extern_crate_mut, ItemExternCrate);
    impl_visit_mut!(visit_item_fn_mut, ItemFn);
    impl_visit_mut!(visit_item_foreign_mod_mut, ItemForeignMod);
    impl_visit_mut!(visit_item_impl_mut, ItemImpl);
    impl_visit_mut!(visit_item_macro_mut, ItemMacro);
    impl_visit_mut!(visit_item_mod_mut, ItemMod);
    impl_visit_mut!(visit_item_static_mut, ItemStatic);
    impl_visit_mut!(visit_item_struct_mut, ItemStruct);
    impl_visit_mut!(visit_item_trait_mut, ItemTrait);
    impl_visit_mut!(visit_item_trait_alias_mut, ItemTraitAlias);
    impl_visit_mut!(visit_item_type_mut, ItemType);
    impl_visit_mut!(visit_item_union_mut, ItemUnion);
    impl_visit_mut!(visit_item_use_mut, ItemUse);
    impl_visit_mut!(visit_lifetime_param_mut, LifetimeParam);
    impl_visit_mut!(visit_local_mut, Local);
    impl_visit_mut!(visit_pat_ident_mut, PatIdent);
    impl_visit_mut!(visit_pat_or_mut, PatOr);
    impl_visit_mut!(visit_pat_paren_mut, PatParen);
    impl_visit_mut!(visit_pat_reference_mut, PatReference);
    impl_visit_mut!(visit_pat_rest_mut, PatRest);
    impl_visit_mut!(visit_pat_slice_mut, PatSlice);
    impl_visit_mut!(visit_pat_struct_mut, PatStruct);
    impl_visit_mut!(visit_pat_tuple_mut, PatTuple);
    impl_visit_mut!(visit_pat_tuple_struct_mut, PatTupleStruct);
    impl_visit_mut!(visit_pat_type_mut, PatType);
    impl_visit_mut!(visit_pat_wild_mut, PatWild);
    impl_visit_mut!(visit_receiver_mut, Receiver);
    impl_visit_mut!(visit_stmt_macro_mut, StmtMacro);
    impl_visit_mut!(visit_trait_item_const_mut, TraitItemConst);
    impl_visit_mut!(visit_trait_item_fn_mut, TraitItemFn);
    impl_visit_mut!(visit_trait_item_macro_mut, TraitItemMacro);
    impl_visit_mut!(visit_trait_item_type_mut, TraitItemType);
    impl_visit_mut!(visit_type_param_mut, TypeParam);
    impl_visit_mut!(visit_variadic_mut, Variadic);
    impl_visit_mut!(visit_variant_mut, Variant);
}
