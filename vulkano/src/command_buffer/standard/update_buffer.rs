// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::mem;
use std::ops::Range;
use std::sync::Arc;

use buffer::Buffer;
use buffer::BufferSlice;
use command_buffer::standard::LatestBufferUsage;
use command_buffer::standard::StdCommandBuffer;
use command_buffer::standard::StdCommandBufferBuilder;
use command_buffer::sys::UnsafeCommandBufferBuilder;
use image::Image;
use image::sys::Layout;
use sync::AccessFlagBits;
use sync::PipelineStages;

use VulkanObject;

/// Wrapper around a `StdCommandBufferBuilder` that adds a buffer updating command at the end of
/// the builder.
pub struct StdUpdateBufferBuilder<'a, T, D: 'a, B> {
    inner: T,
    data: &'a D,
    buffer: Arc<B>,
    flushed: bool,
}

impl<'a, T, D: 'a, B> StdUpdateBufferBuilder<'a, T, D, B> where T: StdCommandBufferBuilder {
    /// Adds the command at the end of `inner`.
    pub fn new<'b, S>(inner: T, buffer: S, data: &'a D) -> StdUpdateBufferBuilder<'a, T, D, B>
        where S: Into<BufferSlice<'b, D, B>>,
              B: Buffer + 'b
    {
        let buffer = buffer.into();

        // FIXME: check outsideness of render pass

        // TODO: return error instead
        assert_eq!(buffer.offset() % 4, 0);
        assert_eq!(buffer.size() % 4, 0);
        assert!(mem::size_of_val(data) <= 65536);
        assert!(buffer.buffer().inner_buffer().usage_transfer_dest());

        // Now that we know the command is valid, we request the right state.
        {
            let stages = PipelineStages { transfer: true, .. PipelineStages::none() };
            let access = AccessFlagBits { transfer_write: true, .. AccessFlagBits::none() };
            inner.transition_buffer_state(buffer, stages, access, true);
        }

        StdUpdateBufferBuilder {
            inner: inner,
            data: data,
            buffer: buffer.buffer().clone(),
            flushed: false,
        }
    }

    fn flush(&mut self) {
        unsafe {
            if self.flushed { return; }
            self.flushed = true;

            self.inner.add_command(|cb| unimplemented!());
        }
    }
}

unsafe impl<'a, T, D: 'a, B> StdCommandBufferBuilder for StdUpdateBufferBuilder<'a, T, D, B>
    where T: StdCommandBufferBuilder,
          B: Buffer
{
    type BuildOutput = StdUpdateBuffer<T::BuildOutput, B>;
    type Pool = T::Pool;

    // The second parameter is whether or not to flush before submitting the barrier.
    type BarrierPrototype = (T::BarrierPrototype, bool);

    #[inline]
    unsafe fn add_command<F>(&mut self, cmd: F)
        where F: FnOnce(&mut UnsafeCommandBufferBuilder<T::Pool>)
    {
        self.flush();
        self.inner.add_command(cmd);
    }

    #[inline]
    unsafe fn buffer_memory_barrier<'a, T: ?Sized, B>(&mut self, buffer: BufferSlice<'a, T, B>,
                                                      src_stages: PipelineStages,
                                                      src_access: AccessFlagBits,
                                                      dest_stages: PipelineStages,
                                                      dest_access: AccessFlagBits, by_region: bool,
                                                      queue_transfer_from: Option<u32>)
    {
        self.flush();
        self.inner.buffer_memory_barrier(buffer, src_stages, src_access, dest_stages, dest_access,
                                         by_region, queue_transfer_from)
    }

    #[inline]
    fn current_image_layout<I>(&self, image: &I, mipmaps: Range<u32>, layers: Range<u32>)
                               -> Option<Layout>
        where I: Image
    {
        self.inner.current_image_layout(image, mipmaps, layers)
    }

    #[inline]
    fn build(mut self) -> StdUpdateBuffer<T::BuildOutput, B> {
        self.flush();

        StdUpdateBuffer {
            inner: self.inner.build(),
            buffer: self.buffer,
        }
    }
}

/// Wrapper around a `StdUpdateBuffer` that adds a buffer updating command at the end of the
/// command buffer.
pub struct StdUpdateBuffer<T, B> {
    inner: T,
    buffer: Arc<B>,
}

unsafe impl<T, B> StdCommandBuffer for StdUpdateBuffer<T, B> where T: StdCommandBuffer {
}
