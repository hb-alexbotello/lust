use crate::config::ImageKind;
use crate::controller::get_bucket_by_id;
use crate::StorageBackend;
use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use uuid::Uuid;

pub struct GCPCloudStorageBackend {
    bucket_name: String,
    client: cloud_storage::Client,
}

impl GCPCloudStorageBackend {
    pub async fn new(
        bucket_name: String,
        service_account_path: String,
    ) -> anyhow::Result<Self> {
        // the cloud_storage crate requires the SERVICE_ACCOUNT env var to be set
        std::env::set_var("SERVICE_ACCOUNT", &service_account_path);

        // read from the bucket to ensure we're authenticated
        // and that the chosen bucket exists
        let client = cloud_storage::Client::default();
        client.bucket().read(&bucket_name).await?;

        Ok(Self {
            bucket_name,
            client,
        })
    }

    #[inline]
    fn format_path(
        &self,
        bucket_id: u32,
        sizing_id: u32,
        image_id: Uuid,
        format: ImageKind,
    ) -> String {
        format!(
            "{}/{}/{}.{}",
            bucket_id,
            sizing_id,
            image_id,
            format.as_file_extension()
        )
    }
}

#[async_trait]
impl StorageBackend for GCPCloudStorageBackend {
    async fn store(
        &self,
        bucket_id: u32,
        image_id: Uuid,
        kind: ImageKind,
        sizing_id: u32,
        data: Bytes,
    ) -> anyhow::Result<()> {
        let store_in = self.format_path(bucket_id, sizing_id, image_id, kind);

        debug!("Storing image in bucket @ {}", &store_in);

        self.client
            .object()
            .create(
                &self.bucket_name,
                data.to_vec(),
                &store_in,
                &kind.as_content_type(),
            )
            .await?;

        Ok(())
    }

    async fn fetch(
        &self,
        bucket_id: u32,
        image_id: Uuid,
        kind: ImageKind,
        sizing_id: u32,
    ) -> anyhow::Result<Option<Bytes>> {
        let store_in = self.format_path(bucket_id, sizing_id, image_id, kind);

        debug!("Retrieving image in bucket @ {}", &store_in);

        let object = self
            .client
            .object()
            .download(&self.bucket_name, &store_in)
            .await?;

        if object.is_empty() {
            return Ok(None);
        }

        Ok(Some(Bytes::from(object)))
    }

    async fn delete(
        &self,
        bucket_id: u32,
        image_id: Uuid,
    ) -> anyhow::Result<Vec<(u32, ImageKind)>> {
        let bucket = get_bucket_by_id(bucket_id)
            .ok_or_else(|| anyhow!("Bucket does not exist."))?
            .cfg();

        let mut hit_entries = vec![];
        for sizing_id in bucket.sizing_preset_ids().iter().copied() {
            for kind in ImageKind::variants() {
                let store_in = self.format_path(bucket_id, sizing_id, image_id, *kind);
                debug!("Purging file in bucket @ {}", &store_in);
                self.client
                    .object()
                    .delete(&self.bucket_name, &store_in)
                    .await?;
                hit_entries.push((sizing_id, *kind));
            }
        }

        Ok(hit_entries)
    }
}
