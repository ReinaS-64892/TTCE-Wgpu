use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::tex_trans_core_engine::TexTransCoreEngineContext;

pub struct TTStorageBuffer {
    pub(crate) buffer: Arc<wgpu::Buffer>,
}

impl TexTransCoreEngineContext<'_> {
    pub fn allocate_storage_buffer(&self, buffer_len: i32, downloadable: bool) -> TTStorageBuffer {
        let label = format!("storage buffer from allocate - Length:{}", buffer_len);
        let alined_len = ((buffer_len + 4) & !3).max(4) as u64;
        let buffer_desc = wgpu::BufferDescriptor {
            label: Some(label.as_str()),
            usage: if downloadable {
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ
            } else {
                wgpu::BufferUsages::STORAGE
            },
            size: alined_len,
            mapped_at_creation: false,
        };

        let buffer = self.engine.device.create_buffer(&buffer_desc);

        TTStorageBuffer {
            buffer: Arc::new(buffer),
        }
    }

    pub fn upload_storage_buffer(
        &self,
        buffer_data_span: &[u8],
        downloadable: bool,
    ) -> TTStorageBuffer {
        let label = format!(
            "storage buffer from upload - Length:{}",
            buffer_data_span.len()
        );
        let buffer_desc = wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            usage: if downloadable {
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ
            } else {
                wgpu::BufferUsages::STORAGE
            },
            contents: buffer_data_span,
        };
        let buffer = self.engine.device.create_buffer_init(&buffer_desc);

        TTStorageBuffer {
            buffer: Arc::new(buffer),
        }
    }

    pub async fn download_storage_buffer(
        &mut self,
        storage_buffer: &TTStorageBuffer,
    ) -> Result<(), wgpu::BufferAsyncError> {
        let rb_buffer_slice = storage_buffer.buffer.slice(..);
        let (sender, receiver) = tokio::sync::oneshot::channel();
        rb_buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            sender.send(v).unwrap();
        });

        self.engine
            .device
            .poll(wgpu::Maintain::wait())
            .panic_on_timeout();

        receiver.await.unwrap()
    }
}
