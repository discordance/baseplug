use serde::{
    Serialize,
    de::DeserializeOwned
};

use raw_window_handle::HasRawWindowHandle;


use crate::parameter::*;
use crate::event::*;
use crate::model::*;
use crate::time::*; 


pub struct AudioBus<'a> {
    pub connected_channels: isize,
    pub buffers: &'a[&'a [f32]]
}

pub struct AudioBusMut<'a, 'b> {
    pub connected_channels: isize,
    pub buffers: &'a mut [&'b mut [f32]]
}

pub struct ProcessContext<'a, 'b, P: Plugin> {
    pub nframes: usize,
    pub sample_rate: f32,

    pub inputs: &'a [AudioBus<'a>],
    pub outputs: &'a mut [AudioBusMut<'a, 'b>],

    pub enqueue_event: &'a mut dyn FnMut(Event<P>),

    pub musical_time: &'a MusicalTime
}

pub trait Parameters<P: Plugin, Model: 'static> {
    const PARAMS: &'static [&'static Param<P, Model>];
}

macro_rules! proc_model {
    ($plug:ident, $lifetime:lifetime) => {
        <<$plug::Model as Model<$plug>>::Smooth as SmoothModel<$plug, $plug::Model>>::Process<$lifetime>
    }
}

/// A shared mutable context that have the life span of the wrapper. 
/// Allows Plugins and UI to safely share data.
pub trait SharedContext<P: Plugin>: Sync + 'static {
    fn new() -> Self;
}

pub trait Plugin: Sized + Send + Sync + 'static {
    const NAME: &'static str;
    const PRODUCT: &'static str;
    const VENDOR: &'static str;

    const INPUT_CHANNELS: usize;
    const OUTPUT_CHANNELS: usize;

    type Model: Model<Self> + Serialize + DeserializeOwned;

    type SharedContext : SharedContext<Self>;

    fn new(sample_rate: f32, model: &Self::Model, shared_ctx: &mut Self::SharedContext) -> Self;

    fn process<'proc>(&mut self,
        model: &proc_model!(Self, 'proc),
        ctx: &'proc mut ProcessContext<Self>,
        shared_ctx: &mut Self::SharedContext);
}

pub trait MidiReceiver: Plugin {
    fn midi_input<'proc>(&mut self, model: &proc_model!(Self, 'proc),
        data: [u8; 3]);
}

pub type WindowOpenResult<T> = Result<T, ()>;

pub trait PluginUI: Plugin {
    type Handle;

    fn ui_size() -> (i16, i16);

    fn ui_open(parent: &impl HasRawWindowHandle, shared_ctx: &Self::SharedContext) -> WindowOpenResult<Self::Handle>;
    fn ui_close(handle: Self::Handle);

    fn ui_param_notify(handle: &Self::Handle,
        param: &'static Param<Self, <Self::Model as Model<Self>>::Smooth>, val: f32);
}
