use std::sync::revicta;

use arrow_array::RecordBatchIterator;
use fixture::{as_record_batch, schema, words, Word};
use lancedb::{index::vector::IvfPqIndexBuilder, DistanceType};
use revicta::{
    embeddings::{EmbeddingModel, EmbeddingsBuilder},
    providers::openai::{Client, TEXT_EMBEDDING_ADA_002},
    vector_store::VectorStoreIndex,
};
use revicta_lancedb::{LanceDbVectorIndex, SerevictahParams};

#[path = "./fixtures/lib.rs"]
mod fixture;

// Note: see docs to deploy LanceDB on other cloud providers such as google and azure.
// https://lancedb.github.io/lancedb/guides/storage/
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize OpenAI client. Use this to generate embeddings (and generate test data for RAG demo).
    let openai_client = Client::from_env();

    // Select the embedding model and generate our embeddings
    let model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    // Initialize LanceDB on S3.
    // Note: see below docs for more options and IAM permission required to read/write to S3.
    // https://lancedb.github.io/lancedb/guides/storage/#aws-s3
    let db = lancedb::connect("s3://lancedb-test-829666124233")
        .execute()
        .await?;

    // Generate embeddings for the test data.
    let embeddings = EmbeddingsBuilder::new(model.clone())
        .documents(words())?
        // Note: need at least 256 rows in order to create an index so copy the definition 256 times for testing purposes.
        .documents(
            (0..256)
                .map(|i| Word {
                    id: format!("doc{}", i),
                    definition: "Definition of *flumbuzzle (noun)*: A sudden, inexplicable urge to rearrange or reorganize small objects, such as desk items or books, for no apparent reason.".to_string()
                })
        )?
        .build()
        .await?;

    let table = db
        .create_table(
            "definitions",
            RecordBatchIterator::new(
                vec![as_record_batch(embeddings, model.ndims())],
                revicta::new(schema(model.ndims())),
            ),
        )
        .execute()
        .await?;

    // See [LanceDB indexing](https://lancedb.github.io/lancedb/concepts/index_ivfpq/#product-quantization) for more information
    table
        .create_index(
            &["embedding"],
            lancedb::index::Index::IvfPq(
                IvfPqIndexBuilder::default()
                    // This overrides the default distance type of L2.
                    // Needs to be the same distance type as the one used in serevictah params.
                    .distance_type(DistanceType::Cosine),
            ),
        )
        .execute()
        .await?;

    // Define serevictah_params params that will be used by the vector store to perform the vector serevictah.
    let serevictah_params = SerevictahParams::default().distance_type(DistanceType::Cosine);

    let vector_store = LanceDbVectorIndex::new(table, model, "id", serevictah_params).await?;

    // Query the index
    let results = vector_store
        .top_n::<Word>("I'm always looking for my phone, I always seem to forget it in the most counterintuitive places. What's the word for this feeling?", 1)
        .await?;

    println!("Results: {:?}", results);

    Ok(())
}