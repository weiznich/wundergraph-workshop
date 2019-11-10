#[macro_export]
macro_rules! from_sql_function {
    (
        $fn_name: ident ($($arg: ident : $arg_ty: ty)*) {
            $($(#[$($meta: tt)+])* $field_name: ident -> $field_ty: ty,)*
        }
    ) => {
        #[allow(dead_code)]
        mod $fn_name {
            use diesel::query_source::*;
            use diesel::expression::*;
            use diesel::query_builder::*;
            use diesel::sql_types::*;
            use wundergraph::helper::NamedTable;

            #[allow(non_camel_case_types)]
            pub struct $fn_name {
                $($arg: std::sync::Arc<dyn diesel::expression::BoxableExpression<(), diesel::pg::Pg, SqlType = $arg_ty>>,)*
            }

            #[allow(non_camel_case_types)]
            pub type table = $fn_name;

            impl Clone for $fn_name {
                fn clone(&self) -> Self {
                    Self {
                        $($arg: self.$arg.clone(),)*
                    }
                }
            }

            impl Table for $fn_name {
                type PrimaryKey = $crate::from_sql_function!(@collect_primary_key [] $($(#[$($meta)*])* $field_name,)*);
                type AllColumns = ($(columns::$field_name,)*);

                fn primary_key(&self) -> Self::PrimaryKey {
                    $crate::from_sql_function!(@collect_primary_key [] $($(#[$($meta)*])* $field_name,)*)
                }

                fn all_columns() -> Self::AllColumns {
                    ($(columns::$field_name,)*)
                }
            }

            impl QueryId for $fn_name {
                type QueryId = ();
                const HAS_STATIC_QUERY_ID: bool = false;
            }

            impl AppearsInFromClause<$fn_name> for $fn_name {
                type Count = Once;
            }

            impl AppearsInFromClause<$fn_name> for () {
                type Count = Never;
            }

            impl AsQuery for $fn_name {
                type SqlType = <<Self as Table>::AllColumns as Expression>::SqlType;
                type Query = SelectStatement<$fn_name>;

                fn as_query(self) -> Self::Query {
                    SelectStatement::simple(self)
                }
            }

            impl QuerySource for $fn_name {
                type FromClause = Self;
                type DefaultSelection = <Self as Table>::AllColumns;

                fn from_clause(&self) -> Self::FromClause {
                    self.clone()
                }

                fn default_selection(&self) -> Self::DefaultSelection {
                    Self::all_columns()
                }
            }

            impl diesel::associations::HasTable for $fn_name {
                type Table = Self;

                fn table() -> Self {
                    Self {
                        $($arg: std::sync::Arc::new(diesel::dsl::sql("")),)*
                    }
                }
            }

            impl QueryFragment<diesel::pg::Pg> for $fn_name {
                #[allow(dead_code, unused_assignments)]
                fn walk_ast(&self, mut pass: AstPass<diesel::pg::Pg>) -> diesel::result::QueryResult<()> {
                    pass.push_sql(stringify!($fn_name));
                    pass.push_sql("(");
                    let mut first = true;
                    $(
                        if first {
                            first = false;
                        } else {
                            pass.push_sql(", ");
                        }
                        self.$arg.walk_ast(pass.reborrow())?;
                    )*
                    pass.push_sql(") AS ");
                    pass.push_sql(stringify!($fn_name));
                    Ok(())
                }
            }

            impl NamedTable for $fn_name {
                fn name(&self) -> std::borrow::Cow<'static, str> {
                    ::std::borrow::Cow::Borrowed(stringify!($fn_name))
                }
            }

            mod columns {
                use diesel::sql_types::*;
                use diesel::prelude::*;
                use diesel::expression::NonAggregate;
                use diesel::query_builder::{QueryFragment, AstPass};

                $(
                    #[derive(Debug, Default, Copy, Clone)]
                    #[allow(non_camel_case_types)]
                    pub struct $field_name;

                    impl diesel::expression::Expression for $field_name {
                        type SqlType = $field_ty;
                    }

                    impl SelectableExpression<super::$fn_name> for $field_name {}
                    impl AppearsOnTable<super::$fn_name> for $field_name {}
                    impl NonAggregate for $field_name {}
                    impl Column for $field_name {
                        type Table = super::$fn_name;
                        const NAME: &'static str = stringify!($field_name);
                    }

                    impl QueryFragment<diesel::pg::Pg> for $field_name {
                        fn walk_ast(&self, mut pass: AstPass<diesel::pg::Pg>) -> diesel::result::QueryResult<()> {
                            pass.push_identifier(stringify!($fn_name))?;
                            pass.push_sql(".");
                            pass.push_identifier(stringify!($field_name))?;
                            Ok(())
                        }
                    }

                    impl<T> diesel::EqAll<T> for $field_name where
                        T: diesel::expression::AsExpression<$field_ty>,
                        diesel::dsl::Eq<$field_name, T>: diesel::Expression<SqlType=diesel::sql_types::Bool>,
                    {
                        type Output = diesel::dsl::Eq<Self, T>;

                        fn eq_all(self, rhs: T) -> Self::Output {
                            diesel::expression::operators::Eq::new(self, rhs.as_expression())
                        }
                    }


                )*
            }

            pub use columns::*;

            #[allow(non_camel_case_types)]
            pub(super) mod function {
                use diesel::sql_types::*;

                pub fn $fn_name<$($arg,)*>($($arg: $arg,)*) -> super::$fn_name
                where $(
                    $arg: diesel::expression::AsExpression<$arg_ty>,
                    <$arg as diesel::expression::AsExpression<$arg_ty>>::Expression: diesel::expression::BoxableExpression<(), diesel::pg::Pg, SqlType = $arg_ty> + 'static,
                )*
                {
                    super::$fn_name {
                        $($arg: std::sync::Arc::new($arg.as_expression()),)*
                    }
                }
            }
        }

        #[allow(dead_code)]
        pub use self::$fn_name::function::$fn_name;

    };

    (@collect_primary_key
     [$($pk: ident,)*]
     $(#[$($meta: tt)*])* #[primary_key] $(#[$($meta2: tt)*])*
     $field: ident, $($rest: tt)*
    ) => {
        $crate::from_sql_function!(@collect_primary_key [$($pk,)* $field] $($rest)*)
    };

    (@collect_primary_key
     [$($pk: ident,)*]
     $(#[$($meta: tt)*])*
     $field: ident, $($rest: tt)*
    ) => {
        $crate::from_sql_function!(@collect_primary_key [$($pk,)*] $($rest)*)
    };
    (@collect_primary_key [$pk: ident,]) => {
        $pk
    };
    (@collect_primary_key [$($pk: ident,)+]) => {
        ($($pk,)*)
    };
    (@collect_primary_key []) => {
        columns::id
    };
}
