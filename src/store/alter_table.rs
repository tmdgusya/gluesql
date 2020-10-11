use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

use sqlparser::ast::ColumnDef;

use crate::result::MutResult;

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum AlterTableError {
    #[error("Column not found")]
    ColumnNotFound,

    #[error("Default value is required: {0}")]
    DefaultValueRequired(String),

    #[error("Column already exists: {0}")]
    ColumnAlreadyExists(String),
}

pub trait AlterTable
where
    Self: Sized,
{
    fn rename_schema(self, table_name: &str, new_table_name: &str) -> MutResult<Self, ()>;

    fn rename_column(
        self,
        table_name: &str,
        old_column_name: &str,
        new_column_name: &str,
    ) -> MutResult<Self, ()>;

    fn add_column(self, table_name: &str, column_def: &ColumnDef) -> MutResult<Self, ()>;
}
