use futures::executor::block_on;
use wgpu::{Device, Instance, Queue, Surface, SwapChain, SwapChainError, SwapChainFrame};
use winit::{dpi::PhysicalSize, window::Window};

/// wgpu device, surface and swap chain for a single window.
pub struct Gpu {
    pub device: Device,
    pub queue: Queue,
    surface: Surface,
    swap_chain: SwapChain,
    _instance: Instance,
}

impl Gpu {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    pub fn new(window: &Window) -> Gpu {
        let instance = Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc(window.inner_size()));

        Gpu {
            device,
            queue,
            surface,
            swap_chain,
            _instance: instance,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &swap_chain_desc(size));
    }

    pub fn current_frame(&mut self) -> Result<SwapChainFrame, SwapChainError> {
        self.swap_chain.get_current_frame()
    }
}

fn swap_chain_desc(size: PhysicalSize<u32>) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: Gpu::FORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    }
}
