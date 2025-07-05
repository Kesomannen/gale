use eyre::{ContextCompat, Result};
use tantivy::{
    collector::TopDocs, query::QueryParser, schema::*, Index, IndexWriter, ReloadPolicy,
    TantivyDocument,
};
use tauri::AppHandle;
use tempfile::TempDir;
use tracing::info;
use uuid::Uuid;

use crate::{state::ManagerExt, thunderstore::BorrowedMod};

pub async fn test(app: AppHandle) {
    tokio::task::spawn_blocking(|| {
        test_inner(app).unwrap();
    });
}

fn test_inner(app: AppHandle) -> Result<()> {
    let index_path = TempDir::new()?;

    let mut schema_builder = Schema::builder();

    schema_builder.add_bytes_field("uuid", STORED);
    schema_builder.add_text_field("owner", TEXT);
    schema_builder.add_text_field("name", TEXT);
    schema_builder.add_text_field("version", TEXT);
    schema_builder.add_text_field("description", TEXT);
    schema_builder.add_text_field("readme", TEXT);

    let schema = schema_builder.build();

    info!("creating index in {}", index_path.path().display());

    let index = Index::create_in_dir(&index_path, schema.clone())?;

    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    let uuid = schema.get_field("uuid").unwrap();
    let owner = schema.get_field("owner").unwrap();
    let name = schema.get_field("name").unwrap();
    let version_field = schema.get_field("version").unwrap();
    let description = schema.get_field("description").unwrap();

    let thunderstore = app.lock_thunderstore();

    for BorrowedMod { package, version } in thunderstore.latest() {
        //info!("writing {}", package.ident);
        let mut doc = TantivyDocument::default();

        doc.add_bytes(uuid, package.uuid.as_bytes());
        doc.add_text(owner, package.owner());
        doc.add_text(name, package.name());
        doc.add_text(version_field, version.version());
        doc.add_text(description, &version.description);

        index_writer.add_document(doc)?;
    }

    drop(thunderstore);

    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;

    let searcher = reader.searcher();
    let query_parser =
        QueryParser::for_index(&index, vec![owner, name, version_field, description]);

    let query = query_parser.parse_query("a")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(1000))?;

    let thunderstore = app.lock_thunderstore();

    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        let uuid = retrieved_doc
            .get_first(uuid)
            .context("uuid field is missing")?
            .as_bytes()
            .unwrap();
        let uuid = Uuid::from_slice(uuid).unwrap();

        let package = thunderstore.get_package(uuid)?;
        let version = package.latest();

        info!("{}: {}", score, version.ident);
    }

    drop(thunderstore);

    Ok(())
}
