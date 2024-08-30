use std::os::raw::c_char;
use std::ffi::CStr;

use wgpu::util::DeviceExt;
use rayon::prelude::*;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct GPUCollection {
    device : Arc<wgpu::Device> ,
    queue : Arc<wgpu::Queue> ,
}
// hashing GPU res
static mut GPU_RES_KEEPER : Option<Arc<Mutex<HashMap<GPUComputingConfig , GPUCollection>>>> = None;





/// NOTE : on linux machines memory leak might happen if you use 
/// vulkan backend until NVIDIA drivers for linux get fixed .
///
///




#[repr(C)]
#[derive(Clone , Debug)]
#[derive(Eq, Hash, PartialEq)]
/// computing backends of the api 
pub enum GPUComputingBackend {
    /// targets all of the backends 
    all = 0 , 
    /// default backend
    default_backend = 1 ,
    /// Supported on Windows, Linux/Android, and macOS/iOS via Vulkan Portability (with the Vulkan feature enabled)
    vulkan = 2,
    /// Supported on Linux/Android, the web through webassembly via WebGL, and Windows and macOS/iOS via ANGLE
    opengl = 3 ,
    /// MacOS & iOS only
    metal = 4 ,
    /// Windows +10 only
    direct_x12 = 5,
    /// browser WebGPU
    webgpu = 6 ,
    /// targets VULKAN METALDX12 BROWSER_WEBGPU
    highest_support = 7 ,
    /// targets OpenGL backend
    lowest_support = 8 ,
}

#[repr(C)]
#[derive(Clone , Debug)]
#[derive(Eq, Hash, PartialEq)]
/// this enum is used to tell to API
/// to setup GPU resources based on power saving rules or
/// not
pub enum GPUPowerSettings {
    /// power and performance does not matter
    none = 0 ,
    /// choose based on the power saving rules
    LowPower = 1 ,
    /// performance is more important
    HighPerformance = 2 ,
}

#[repr(C)]
#[derive(Clone , Debug)]
#[derive(Eq, Hash, PartialEq)]
/// this enum affects speed of the api 
/// by setting how much gpu resources
/// are needed directly , if you take 
/// too much which your hardware cannot 
/// provide , panic happens
pub enum GPUSpeedSettings {
    /// the lowest resources , supported on all backends
    lowest_speed = 0 ,
    /// low resources , supported on all backends expect webgl2
    /// which our api does not aim to support for now 
    low_speed = 1 ,
    /// the default
    default_speed = 2 ,
    /// will be supported in next versions , for now it is equal to 
    /// low_speed
    custom_speed = 3 ,
}

#[repr(C)]
#[derive(Clone , Debug)]
#[derive(Eq, Hash, PartialEq)]
/// this settings used to tell gpu pre information about 
/// our work 
pub enum GPUMemorySettings {
    /// our app needs to me more performant instead being 
    /// cable of allocating too much memory on gpu side
    prefer_performance = 0 ,
    /// our app will need to allocate memory on gpu side 
    prefer_memory = 1 ,
    /// will be supported in next versions , by default for now it is set to 
    /// prefer_memory . in next versions you can tell how much memory 
    /// you need to allocate on gpu side
    custom_memory = 3 ,
}

#[repr(C)]
#[derive(Debug, Clone)]
#[derive(Eq, Hash, PartialEq)]
/// as config field you have to provide GPUComputingConfig which
/// represent settings which you wanted
pub struct GPUComputingConfig {
    /// set backend which you want 
    pub backend : GPUComputingBackend ,
    /// set power settings which meets your needs 
    pub power : GPUPowerSettings ,
    /// set speed settings which matches your needs
    pub speed : GPUSpeedSettings ,
    /// tell to gpu about your memory usage 
    pub memory : GPUMemorySettings ,
}

#[repr(C)]
#[derive(Debug, Clone)]
/// CKernel which will represent your GPU task
/// like how Manifest.xml does in an android 
/// project
pub struct CKernel {
    /// set max number of workgroups in x dimension
    pub x : u32 ,
    /// set max number of workgroups in y dimension
    pub y : u32 ,
    /// set max number of workgroups in z dimension
    pub z : u32 ,
    /// this is a kernel code which must be in wgsl for now
    /// more shading languages will be supported in the future
    pub code : *const c_char ,
    /// this part in the code , tell to the api which 
    /// function in the code must be called by gpu 
    /// when the task is sent to gpu 
    pub code_entry_point : *const c_char ,
    /// by setting config you can customize behavior of the 
    /// gpu
    pub config : GPUComputingConfig ,

    index : usize ,
}

#[no_mangle]
/// because setting CKernel config can be annoying if you just 
/// want to do simple task , this function provides general 
/// config which will meet most of your needs
pub extern "C" fn set_kernel_default_config(kernel: *mut CKernel) {
    // println!("set start"); 
    if kernel.is_null() {
        return;
    }

    unsafe {

        let kernel = &mut *kernel;


        kernel.config = GPUComputingConfig {
            backend: GPUComputingBackend::opengl,
            power: GPUPowerSettings::none,
            speed: GPUSpeedSettings::low_speed,
            memory: GPUMemorySettings::prefer_memory,
        };


    }

    // println!("set done");
}


impl CKernel {
    // this function is for future implementions
    fn code_as_string(&self) -> Option<String> {
        unsafe {
            if self.code.is_null() {
                None    
            } else {
                Some(CStr::from_ptr(self.code).to_string_lossy().into_owned())
            }
        }
    }

    // this function is for future implementions
    fn ep_as_string(&self) -> Option<String> {
        unsafe {
            if self.code.is_null() {
                None    
            } else {
                Some(CStr::from_ptr(self.code_entry_point).to_string_lossy().into_owned())
            }
        }
    }

    // this function converts enums to
    // equivalent gpu resources
    fn get_real_config(&self) -> (Arc<wgpu::Device> , Arc<wgpu::Queue>) {

        unsafe{
            match &GPU_RES_KEEPER {
                None => {
                    GPU_RES_KEEPER = Some(Arc::new(Mutex::new(HashMap::new())));
                },
                Some(arci) => {
                    let mut GPU_Data = arci.lock().unwrap();
                    if GPU_Data.contains_key(&self.config) {
                        return (Arc::clone(&GPU_Data[&self.config].device) , Arc::clone(&GPU_Data[&self.config].queue));
                    }
                }
            }
        }
        // println!("get real start");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{
            backends : match self.config.backend {
                GPUComputingBackend::vulkan => {
                    wgpu::Backends::VULKAN
                },
                GPUComputingBackend::opengl => {
                    wgpu::Backends::GL
                },
                GPUComputingBackend::all => {
                    wgpu::Backends::all()
                },
                GPUComputingBackend::default_backend => {
                    wgpu::Backends::default()
                },
                GPUComputingBackend::metal => {
                    wgpu::Backends::METAL 
                },
                GPUComputingBackend::direct_x12 => {
                    wgpu::Backends::DX12
                },
                GPUComputingBackend::highest_support => {
                    wgpu::Backends::PRIMARY
                },
                GPUComputingBackend::lowest_support => {
                    wgpu::Backends::SECONDARY
                },
                GPUComputingBackend::webgpu => {
                    wgpu::Backends::BROWSER_WEBGPU
                },
            },
            ..Default::default()
        });

        let adapter = pollster::block_on(instance
            .request_adapter(&wgpu::RequestAdapterOptions{
                power_preference : match self.config.power {
                    GPUPowerSettings::none => {
                        wgpu::PowerPreference::None
                    },
                    GPUPowerSettings::LowPower => {
                        wgpu::PowerPreference::LowPower
                    },
                    GPUPowerSettings::HighPerformance => {
                        wgpu::PowerPreference::HighPerformance
                    },
                },
                ..Default::default()
            }))
        .expect("ERROR : failed to get adapter");

        let (device, queue) = pollster::block_on(adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: match self.config.speed {
                        GPUSpeedSettings::lowest_speed => {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        },
                        GPUSpeedSettings::low_speed => {
                            wgpu::Limits::downlevel_defaults()
                        },
                        GPUSpeedSettings::custom_speed => {
                            // for now it will be set to downlevel_defaults as placeholderi
                            wgpu::Limits::downlevel_defaults()
                        },
                        GPUSpeedSettings::default_speed => {
                            wgpu::Limits::default()
                        },
                    },
                    memory_hints: match self.config.memory {
                        GPUMemorySettings::prefer_performance => {
                            wgpu::MemoryHints::Performance
                        },
                        GPUMemorySettings::prefer_memory => {
                            wgpu::MemoryHints::MemoryUsage
                        },
                        GPUMemorySettings::custom_memory => {
                            // for now it will be set to MemoryUsage as placeholder
                            wgpu::MemoryHints::MemoryUsage
                        },
                    },
                },
                None,
                ))
                    .expect("ERROR : Adapter could not find the device");

        // println!("get real done");
        unsafe{
            let device = Arc::new(device);
            let queue = Arc::new(queue);
            let arci = GPU_RES_KEEPER.clone().unwrap();
            let mut GPU_Data = arci.lock().unwrap();

            GPU_Data.insert(self.config.clone() , GPUCollection{
                device : Arc::clone(&device) ,
                queue : Arc::clone(&queue) ,
            });

            return (Arc::clone(&device) , Arc::clone(&queue));
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
/// this struct is for passing
/// data based on its bind index 
/// in gpu side 
pub struct DataBinder {
    /// bind index of data in gpu side
    pub bind: u32,
    /// because data must be in uint8_t (u8 in Rust) 
    /// in C you have to pass the data len this way 
    /// 
    /// sizeof(your type) * real_len_of_your_array / sizeof(uint8_t)
    pub data_len: usize,
    /// pointer to your data  in memory , it must be 
    /// uint8_t* (*mut u8 in Rust side) 
    /// in gpu side the type of this data will 
    /// be set based on CKernel code you provided
    pub data: *mut u8,
}

impl DataBinder {
    // this function is for future implementions
    unsafe fn data_as_vec(&self) -> Option<Vec<u8>> {
        if self.data.is_null() {
            None
        } else {
            // Create a Vec<u8> that shares the memory but doesn't deallocate it.
            Some(Vec::from_raw_parts(self.data, self.data_len, self.data_len))
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
/// all DataBinder types which have 
/// the same @group index in your kernel
/// code must all be gathered in this 
/// type
pub struct GroupOfBinders {
    /// index of group in your kernel code 
    pub group : u32 ,
    /// pointer to array which all of the 
    /// DataBinders from same group 
    /// are gathered in 
    pub datas : *mut DataBinder ,
    /// len of datas array
    pub datas_len : usize ,
}



#[no_mangle]
/// the simple and compact function for sending 
/// your computing task to the gpu side
///
/// kernel para = CKernel type which acts as Manifest for your gpu task 
/// data_for_gpu = pointer to array of GroupOfBinders which contains data which must be sent to gpu 
/// gpu_data_len = len of the array of the GroupOfBinders
///
/// unlike CUDA , you dont need to copy data to gpu manually , this function does it for you 
/// in the most performant possible way 
///
/// if you find any bug or any problem , help us to fix it -> https://github.com/SkillfulElectro/EMCompute.git
pub extern "C" fn compute(kernel : CKernel , data_for_gpu : *mut GroupOfBinders , gpu_data_len : usize) -> i32 {

    {
        // println!("compute start");


        if data_for_gpu.is_null(){
            println!("ERROR : data_for_gpu arg of compute function is NULL");
            return -1;
        }

        let (device , queue) = kernel.get_real_config();

        // println!("compute data stage");


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(kernel.code_as_string().expect("ERROR : No computing kernel code provided , code field is not set .").into()),
        });

        // println!("compute pipeline stage");

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: &kernel.ep_as_string().expect("ERROR : No code_entry_point field is set , it must be name of function which your kernel code starts from") ,
            compilation_options: Default::default(),
            cache: None,
        });

        // println!("compute after pipeline");

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut staging_buffers : Vec<wgpu::Buffer> = Vec::new();
        let mut sizes : Vec<wgpu::BufferAddress> = Vec::new();
        let mut storage_buffers : Vec<wgpu::Buffer> = Vec::new();


        let groups : &mut [GroupOfBinders] = unsafe { std::slice::from_raw_parts_mut(data_for_gpu , gpu_data_len) };

        // println!("before cpass");
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            for group in &mut *groups {
                let bind_group_layout = compute_pipeline.get_bind_group_layout(group.group);
                if group.datas.is_null() {
                    println!("ERROR : no data provided for datas field in data_for_gpu arg");
                    return -1;
                }

                let bindings : &mut [DataBinder] = unsafe{
                    std::slice::from_raw_parts_mut(group.datas , group.datas_len)
                };

                let mut tmp_staging_buffers : Vec<wgpu::Buffer> = Vec::new();
                let mut tmp_sizes : Vec<wgpu::BufferAddress> = Vec::new();
                let mut tmp_storage_buffers : Vec<wgpu::Buffer> = Vec::new();

                let mut entries : Vec<wgpu::BindGroupEntry> = Vec::new();

                for binder in &mut *bindings {
                    if binder.data.is_null() {
                        println!("ERROR : null data field in DataBinder found");
                        return -1;
                    }

                    let data : &[u8] = unsafe{
                        std::slice::from_raw_parts(binder.data , binder.data_len)
                    };

                    let size = std::mem::size_of_val(data) as wgpu::BufferAddress;

                    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: None ,
                        size : size ,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });

                    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Storage Buffer"),
                        contents: data ,
                        usage: wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                    });



                    tmp_sizes.push(size);
                    tmp_staging_buffers.push(staging_buffer);
                    tmp_storage_buffers.push(storage_buffer);
                }



                for (i, binder) in bindings.iter().enumerate() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: binder.bind.clone(),
                        resource: tmp_storage_buffers[i].as_entire_binding(),
                    });
                }

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: entries.as_slice() ,
                });

                cpass.set_pipeline(&compute_pipeline);
                cpass.set_bind_group(group.group , &bind_group, &[]);

                storage_buffers.append(&mut tmp_storage_buffers);
                staging_buffers.append(&mut tmp_staging_buffers);
                sizes.append(&mut tmp_sizes);
            }

            cpass.insert_debug_marker("debug_marker");
            cpass.dispatch_workgroups(kernel.x, kernel.y, kernel.z);
        }
        // println!("after cpass");


        for (index, storage_buffer) in storage_buffers.iter().enumerate() {
            encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffers[index], 0, sizes[index]);
        }

        queue.submit(Some(encoder.finish()));


        let mut index : usize = 0;
        for group in groups {
            let bindings : &mut [DataBinder] = unsafe{
                std::slice::from_raw_parts_mut(group.datas , group.datas_len)
            };

            for binder in bindings {
                let data : &mut [u8] = unsafe{
                    std::slice::from_raw_parts_mut(binder.data , binder.data_len)
                };

                let buffer_slice = staging_buffers[index].slice(..);
                let (sender, receiver) = flume::bounded(1);
                buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

                device.poll(wgpu::Maintain::wait()).panic_on_timeout();

                if let Ok(Ok(())) = pollster::block_on(receiver.recv_async()) {
                    let mapped_data = buffer_slice.get_mapped_range();





                    data.par_iter_mut().zip(mapped_data.par_iter()).for_each(|(d, &value)| {
                        *d = value;
                    });



                    drop(mapped_data);
                    staging_buffers[index].unmap();

                } else {
                    panic!("failed to run compute on gpu!")
                }

                index += 1;
            }
        }


        // println!("compute done");

        return 0;
    }
}
