// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Standard implementation of the `CommandBuffer` trait.
//! 
//! Everything in this module is dedicated to the "standard" implementation of command buffers.

use std::ops::Range;
use std::sync::Arc;

use buffer::Buffer;
use buffer::BufferSlice;
use command_buffer::pool::CommandPool;
use command_buffer::sys::UnsafeCommandBufferBuilder;
use image::Image;
use image::sys::Layout;
use sync::AccessFlagBits;
use sync::PipelineStages;

use self::update_buffer::StdUpdateBufferBuilder;

pub mod primary;
pub mod update_buffer;

///
///
/// # How to use
///
/// In order to successfully add a command to a builder that implements this trait, you must:
///
/// - Add a pipeline barrier if necessary.   TODO: clarify
/// - Call `add_command` to add your command to the underlying unsafe builder.
/// - Somehow keep the objects alive for as long as the command buffer is alive, and properly
///   handle CPU-GPU and inter-queue synchronization.
///
/// # How to implement
///
/// Implementing this trait on a simple wrapper around a builder is very straight-forward. Just
/// don't forget to add an initial and a final pipeline barrier if needed.
///
/// Implementing this trait on a wrapper that itself adds a command in the process is also
/// straight-forward, with the additional note that you should keep the objects alive and provide
/// a `BuildOutput` object that correctly handles synchronization.
///
/// However for performance reasons you may not want to implement this trait on a straight-forward
/// way. Instead of directly adding the new barrier and the new command as soon as the wrapper is
/// created, it should keep them in memory instead. When `add_barrier` is called, it should try
/// merge the new barrier with the existing one. If the barriers can be merged, do nothing more.
/// If the barriers can't be merged, call `add_barrier` on the wrapper object with the old barrier
/// followed with `add_command`, and stop filtering calls to `add_barrier` altogether. When
/// `add_command` or `build` is called, flush your own barrier and command immediately.
///
pub unsafe trait StdCommandBufferBuilder {
    /// The finished command buffer.
    type BuildOutput: StdCommandBuffer;

    /// The command pool that was used to build the command buffer.
    type Pool: CommandPool;

    type ResourcesDependencies: ResourcesDependencies;

    /// Adds a buffer update command at the end of the command buffer builder.
    #[inline]
    fn update_buffer<'a, 'b, D: 'a, S, B: 'b>(self, buffer: S, data: &'a D)
                                              -> StdUpdateBufferBuilder<'a, Self, D, B>
        where Self: Sized,
              B: Buffer,
              S: Into<BufferSlice<'b, D, B>>
    {
        StdUpdateBufferBuilder::new(self, buffer, data)
    }

    /// Obtains a temporary access to the command buffer builder in order to add one or multiple
    /// commands to it.
    ///
    /// The implementation **must** call the closure with a correct reference to the builder.
    /// Failure to do so is unsound.
    ///
    /// For performance reasons, you are encouraged to use the barrier-related functions of the
    /// trait in order to add a pipeline barrier, but adding a barrier through `add_command` is
    /// not forbidden.
    // TODO: remove this function and replace it with unsafe equivalents
    unsafe fn add_command<F>(&mut self, cmd: F)
        where F: FnOnce(&mut UnsafeCommandBufferBuilder<Self::Pool>);

    /// Appends a barrier at the end of builder.
    ///
    /// Implementations should batch the calls to the barrier-related functions and flush them
    /// at once when necessary.
    unsafe fn buffer_memory_barrier<'a, T: ?Sized, B>(&mut self, buffer: BufferSlice<'a, T, B>,
                                                      src_stages: PipelineStages,
                                                      src_access: AccessFlagBits,
                                                      dest_stages: PipelineStages,
                                                      dest_access: AccessFlagBits, by_region: bool,
                                                      queue_transfer_from: Option<u32>);

    /// Requires that a specific slice of a buffer be transitionned in order to be available for
    /// the given stages and the given accesses.
    ///
    /// TODO: note about state not being preserved
    unsafe fn transition_buffer_state<'a, T: ?Sized, B>
              (&mut self, buffer: BufferSlice<'a, T, B>, dest_stage: PipelineStages,
               dest_access: AccessFlagBits, by_region: bool)
        where B: Buffer;

    /// Requires that a specific subresource of an image be transitionned in order to be available
    /// for the given stages and the given accesses.
    ///
    /// TODO: note about state not being preserved
    unsafe fn transition_image_state<I>(&mut self, image: &Arc<I>, mipmaps: Range<u32>,
                                        layers: Range<u32>, dest_stage: PipelineStages,
                                        dest_access: AccessFlagBits, by_region: bool,
                                        new_layout: Layout)
        where I: Image;

    /// If the given image is used earlier in the pipeline, this function should return the layout
    /// it is currently in.
    fn current_image_layout<I>(&self, image: &I, mipmaps: Range<u32>, layers: Range<u32>)
                               -> Option<Layout>
        where I: Image;

    /// Returns true if the parameter is the same pipeline as the one that is currently binded on
    /// the graphics slot.
    ///
    /// Since this is purely an optimization to avoid having to bind the pipeline again, you can
    /// return `false` when in doubt.
    ///
    /// This function doesn't take into account any possible command that you add through
    /// `add_command`.
    #[inline]
    fn is_current_graphics_pipeline(&self /*, pipeline: &P */) -> bool {
        false
    }

    /// Finishes building the command buffer.
    ///
    /// Consumes the builder and returns an implementation of `StdCommandBuffer`.
    fn build(self) -> Self::BuildOutput;
}

pub unsafe trait StdCommandBuffer/*: CommandBuffer*/ {
    type TransitionCommandBuffer: CommandBuffer;

    fn build_required_transitions(&self) -> Option<Self::TransitionCommandBuffer>;
}
