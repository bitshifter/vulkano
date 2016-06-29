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
use command_buffer::pool::CommandPool;
use command_buffer::pool::StandardCommandPool;
use command_buffer::standard::ResourcesDependencies;
use command_buffer::standard::StdCommandBuffer;
use command_buffer::standard::StdCommandBufferBuilder;
use command_buffer::standard::StdCommandBufferBuilderRef;
use command_buffer::sys::Flags;
use command_buffer::sys::Kind;
use command_buffer::sys::PipelineBarrierBuilder;
use command_buffer::sys::UnsafeCommandBuffer;
use command_buffer::sys::UnsafeCommandBufferBuilder;
use framebuffer::EmptySinglePassRenderPass;
use image::Image;
use image::sys::Layout;
use sync::PipelineStages;
use sync::AccessFlagBits;

pub struct StdPrimaryCommandBufferBuilder<P = Arc<StandardCommandPool>> where P: CommandPool {
    inner: UnsafeCommandBufferBuilder<P>,
    staging_barrier: PipelineBarrierBuilder,
}

impl<P> StdPrimaryCommandBufferBuilder<P> where P: CommandPool {
    pub fn new(pool: P) -> StdPrimaryCommandBufferBuilder<P> {
        let kind = Kind::Primary::<EmptySinglePassRenderPass, EmptySinglePassRenderPass>;
        let cb = UnsafeCommandBufferBuilder::new(pool, kind, Flags::SimultaneousUse).unwrap();  // TODO: allow handling this error

        StdPrimaryCommandBufferBuilder {
            inner: cb,
            staging_barrier: PipelineBarrierBuilder::new(),
        }
    }
}

unsafe impl<P> StdCommandBufferBuilder for StdPrimaryCommandBufferBuilder<P> where P: CommandPool {
    type BuildOutput = StdPrimaryCommandBuffer<P>;
    type Pool = P;
    type ResourcesDependencies = PipelineBarrierBuilder;

    #[inline]
    unsafe fn add_command<F>(&mut self, cmd: F)
        where F: FnOnce(&mut UnsafeCommandBufferBuilder<P>)
    {
        if !staging_barrier.is_empty() {
            self.inner.pipeline_barrier(mem::replace(&mut self.staging_barrier,
                                                     PipelineBarrierBuilder::new()));
        }

        cmd(&mut self.inner)
    }

    #[inline]
    unsafe fn buffer_memory_barrier<'a, T: ?Sized, B>(&mut self, buffer: BufferSlice<'a, T, B>,
                                                      src_stages: PipelineStages,
                                                      src_access: AccessFlagBits,
                                                      dest_stages: PipelineStages,
                                                      dest_access: AccessFlagBits, by_region: bool,
                                                      queue_transfer_from: Option<u32>)
    {
        self.staging_barrier.add_buffer_memory_barrier(buffer, src_stages, src_access, dest_stages,
                                                       dest_access, by_region, queue_transfer_from);
    }

    #[inline]
    fn build(self) -> StdPrimaryCommandBuffer<P> {
        // TODO: final image transitions

        StdPrimaryCommandBuffer {
            inner: self.inner.build().unwrap(),     // TODO: allow handling this error
        }
    }
}

pub struct StdPrimaryCommandBuffer<P = Arc<StandardCommandPool>> where P: CommandPool {
    inner: UnsafeCommandBuffer<P>
}

unsafe impl<P> StdCommandBuffer for StdPrimaryCommandBuffer<P> where P: CommandPool {
}
