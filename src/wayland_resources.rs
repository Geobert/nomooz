use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::wlr_layer::{LayerShell, LayerSurface},
    shm::{Shm, slot::SlotPool},
};

use crate::virtual_pointer::VirtualPointerManager;

pub struct WaylandResources {
    pub compositor: CompositorState,
    pub layer_shell: LayerShell,
    pub shm: Shm,
    pub virtual_pointer_manager: VirtualPointerManager,
    pub pool: SlotPool,
    pub layer: LayerSurface,
}
