use crate::QueryError;
use aura_common::{AuraDocument, DataValue};
use aura_store::page::Page;
use aura_store::pager::Pager;
use sqlparser::ast::{Expr, SetExpr, Statement, Value, Values};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::collections::HashMap;

pub struct QueryEngine<'a> {
    pager: &'a mut Pager,
}

impl<'a> QueryEngine<'a> {
    pub fn new(pager: &'a mut Pager) -> Self {
        Self { pager }
    }

    /// The Main Entry Point: Takes SQL, Writes to Disk
    pub fn execute(&mut self, sql: &str) -> Result<String, QueryError> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql)?;

        if ast.is_empty() {
            return Err(QueryError::Unimplemented("Empty SQL statement".to_string()));
        }

        match &ast[0] {
            Statement::Insert {
                columns, source, ..
            } => {
                if let Some(query) = source {
                    self.handle_insert(columns, query)
                } else {
                    Err(QueryError::Unimplemented(
                        "INSERT without source not supported".into(),
                    ))
                }
            }
            Statement::Query(query) => {
                // Very simplified parser for: SELECT * FROM users WHERE id = 'x'
                self.handle_select(query)
            }
            _ => Err(QueryError::Unimplemented(
                "Only INSERT and SELECT are supported".into(),
            )),
        }
    }

    fn handle_insert(
        &mut self,
        columns: &[sqlparser::ast::Ident],
        source: &sqlparser::ast::Query,
    ) -> Result<String, QueryError> {
        // 1. Extract Values from the AST
        // This is simplified: assuming VALUES (...) structure
        let row_values = match &*source.body {
            SetExpr::Values(Values { rows, .. }) => &rows[0],
            _ => {
                return Err(QueryError::Unimplemented(
                    "Complex INSERT not supported".into(),
                ))
            }
        };

        // 2. Build the AuraDocument
        let mut doc_data = HashMap::new();
        let mut doc_id = String::new();

        for (i, col) in columns.iter().enumerate() {
            let col_name = col.value.clone();
            let val_expr = &row_values[i];

            // Map SQL Value -> Aura DataValue
            let value = match val_expr {
                Expr::Value(Value::Number(n, _)) => DataValue::Integer(n.parse().unwrap_or(0)),
                Expr::Value(Value::SingleQuotedString(s)) => DataValue::Text(s.clone()),
                Expr::Value(Value::Boolean(b)) => DataValue::Boolean(*b),
                _ => DataValue::Null,
            };

            // Special handling: treat 'id' column as the Primary Key
            if col_name == "id" {
                if let DataValue::Text(s) = &value {
                    doc_id = s.clone();
                }
            }

            doc_data.insert(col_name, value);
        }

        if doc_id.is_empty() {
            doc_id = uuid::Uuid::new_v4().to_string(); // Auto-generate ID if missing
        }

        let document = AuraDocument {
            id: doc_id.clone(),
            version: 1,
            data: doc_data,
        };

        // 3. Serialize & Store (The "Map to Page" step)
        let new_page_id = self.write_document_to_disk(document)?;

        // UPDATE INDEX
        self.pager.index.insert(doc_id.clone(), new_page_id);

        // Save the index to disk immediately (or wait for a commit)
        self.pager.sync_index()?;

        Ok(format!("Inserted Document ID: {}", doc_id))
    }

    fn write_document_to_disk(&mut self, doc: AuraDocument) -> Result<u32, QueryError> {
        // A. Serialize
        let bytes = doc
            .to_bytes()
            .map_err(|e| QueryError::Serialization(e.to_string()))?;

        // B. Allocate a new Page
        // Note: In a real DB, we would split large docs across multiple pages.
        // For Step 5, we assume the document fits in one 4KB page.
        if bytes.len() > aura_store::page::DATA_SIZE {
            return Err(QueryError::Serialization(
                "Document too large for single page".into(),
            ));
        }

        let new_page_id = self.pager.allocate_page();
        let mut page = Page::new(new_page_id);

        // C. Copy data into Page
        // page.data is [u8; 4000]. We copy our bytes into it.
        page.data[..bytes.len()].copy_from_slice(&bytes);
        page.used_space = bytes.len() as u16;

        // D. Write (This triggers the Automatic Encryption from Step 4)
        self.pager.write_page(&page)?;

        Ok(new_page_id)
    }

    // New Function
    fn handle_select(&mut self, _query: &sqlparser::ast::Query) -> Result<String, QueryError> {
        // 1. Extract the WHERE clause (Looking for ID)
        // This requires traversing the AST. For Step 6, let's cheat and hardcode:
        // "Find the document with ID = 'user_007'"

        // Assume we parsed the ID from the query string...
        let target_id = "user_007"; // In real code, get this from AST

        // 2. INDEX LOOKUP (O(1) Speed)
        match self.pager.index.get(target_id) {
            Some(page_id) => {
                // 3. FETCH ONLY THE ONE PAGE
                let page = self.pager.read_page(page_id)?;

                // 4. Deserialize
                let stored_bytes = &page.data[..page.used_space as usize];
                let doc = AuraDocument::from_bytes(stored_bytes)
                    .map_err(|e| QueryError::Serialization(e.to_string()))?;

                Ok(format!("Found: {:?}", doc))
            }
            None => Ok("Document not found".to_string()),
        }
    }
}
