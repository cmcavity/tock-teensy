//! Implementation of the MK66 memory protection unit.
//!
//! - Author: Conor McAvity <cmcavity@stanford.edu>

use kernel::common::regs::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::mpu::{self, Permissions};

#[repr(C)]
struct MpuErrorRegisters {
    ear: ReadOnly<u32, ErrorAddress::Register>,
    edr: ReadOnly<u32, ErrorDetail::Register>,
}

#[repr(C)]
struct MpuRegionDescriptor {
    rgd_word0: ReadWrite<u32, RegionDescriptorWord0::Register>,
    rgd_word1: ReadWrite<u32, RegionDescriptorWord1::Register>,
    rgd_word2: ReadWrite<u32, RegionDescriptorWord2::Register>,
    rgd_word3: ReadWrite<u32, RegionDescriptorWord3::Register>,
}

#[repr(C)]
struct MpuAlternateAccessControl( 
    ReadWrite<u32, RegionDescriptorWord2::Register>
);


/// MPU registers for the K66
///
/// Described in section 22.4 of
/// <https://www.nxp.com/docs/en/reference-manual/K66P144M180SF5RMV2.pdf>
#[repr(C)]
struct MpuRegisters {
    cesr: ReadWrite<u32, ControlErrorStatus::Register>,
    _reserved0: [u32; 3],
    ers: [MpuErrorRegisters; 5],
    _reserved1: [u32; 242],
    rgds: [MpuRegionDescriptor; 12],
    _reserved2: [u32; 208],
    rgdaacs: [MpuAlternateAccessControl; 12],
}

register_bitfields![u32,
    ControlErrorStatus [
        /// Slave Port 0 Error
        SP0ERR OFFSET(31) NUMBITS(1) [],
        /// Slave Port 1 Error
        SP1ERR OFFSET(30) NUMBITS(1) [],
        /// Slave Port 2 Error
        SP2ERR OFFSET(29) NUMBITS(1) [],
        /// Slave Port 3 Error
        SP3ERR OFFSET(28) NUMBITS(1) [],
        /// Slave Port 4 Error
        SP4ERR OFFSET(27) NUMBITS(1) [],
        /// Hardware Revision Level
        HRL OFFSET(16) NUMBITS(4) [],
        /// Number Of Slave Ports
        NSP OFFSET(12) NUMBITS(4) [],
        /// Number Of Region Descriptors
        NRGD OFFSET(8) NUMBITS(4) [
            Eight = 0,
            Twelve = 1,
            Sixteen = 2
        ],
        /// Valid
        VLD OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    ErrorAddress [
        /// Error Address
        EADDR OFFSET(0) NUMBITS(32) []
    ],

    ErrorDetail [
        /// Error Access Control Detail
        EACD OFFSET(16) NUMBITS(16) [],
        /// Error Process Identification
        EPID OFFSET(8) NUMBITS(8) [],
        /// Error Master Number
        EMN OFFSET(4) NUMBITS(4) [],
        /// Error Attributes
        EATTR OFFSET(1) NUMBITS(3) [
            UserModeInstructionAccess = 0,
            UserModeDataAccess = 1,
            SupervisorModeInstructionAccess = 2,
            SupervisorModeDataAccess = 3
        ],
        /// Error Read/Write
        ERW OFFSET(1) NUMBITS(1) [
            Read = 0,
            Write = 1
        ]
    ],

    RegionDescriptorWord0 [
        /// Start Address
        SRTADDR OFFSET(5) NUMBITS(27) []
    ],

    RegionDescriptorWord1 [
        /// End Address
        ENDADDR OFFSET(5) NUMBITS(27) []
    ],

    RegionDescriptorWord2 [
        /// Bus Master 7 Read Enable
        M7RE OFFSET(31) NUMBITS(1) [],
        /// Bus Master 7 Write Enable
        M7WE OFFSET(30) NUMBITS(1) [],
        /// Bus Master 6 Read Enable
        M6RE OFFSET(29) NUMBITS(1) [],
        /// Bus Master 6 Write Enable
        M6WE OFFSET(28) NUMBITS(1) [],
        /// Bus Master 5 Read Enable
        M5RE OFFSET(27) NUMBITS(1) [],
        /// Bus Master 5 Write Enable
        M5WE OFFSET(26) NUMBITS(1) [],
        /// Bus Master 4 Read Enable
        M4RE OFFSET(25) NUMBITS(1) [],
        /// Bus Master 4 Write Enable
        M4WE OFFSET(24) NUMBITS(1) [],
        /// Bus Master 3 Process Identifier Enable
        M3PE OFFSET(23) NUMBITS(1) [],
        /// Bus Master 3 Supervisor Mode Access Control
        M3SM OFFSET(21) NUMBITS(2) [
            ReadWriteExecute = 0,
            ReadExecuteOnly = 1,
            ReadWriteOnly = 2,
            SameAsUserMode = 3 
        ],
        /// Bus Master 3 User Mode Access Control
        M3UM OFFSET(18) NUMBITS(3) [],
        /// Bus Master 2 Process Identifier Enable
        M2PE OFFSET(17) NUMBITS(1) [],
        /// Bus Master 2 Supervisor Mode Access Control
        M2SM OFFSET(15) NUMBITS(2) [
            ReadWriteExecute = 0,
            ReadExecuteOnly = 1,
            ReadWriteOnly = 2,
            SameAsUserMode = 3 
        ],
        /// Bus Master 2 User Mode Access Control 
        M2UM OFFSET(12) NUMBITS(3) [],
        /// Bus Master 1 Process Identifier Enable
        M1PE OFFSET(11) NUMBITS(1) [],
        /// Bus Master 1 Supervisor Mode Access Control
        M1SM OFFSET(9) NUMBITS(2) [
            ReadWriteExecute = 0,
            ReadExecuteOnly = 1,
            ReadWriteOnly = 2,
            SameAsUserMode = 3 
        ],
        /// Bus Master 1 User Mode Access Control
        M1UM OFFSET(6) NUMBITS(3) [],
        /// Bus Master 0 Process Identifier Enable
        M0PE OFFSET(5) NUMBITS(1) [],
        /// Bus Master 0 Supervisor Mode Access Control
        M0SM OFFSET(3) NUMBITS(2) [
            ReadWriteExecute = 0,
            ReadExecuteOnly = 1,
            ReadWriteOnly = 2,
            SameAsUserMode = 3 
        ],
        /// Bus Master 0 User Mode Access Control 
        M0UM OFFSET(0) NUMBITS(3) []
    ],

    RegionDescriptorWord3 [
        /// Process Identifier
        PID OFFSET(24) NUMBITS(8) [],
        /// Process Identifier Mask
        PIDMASK OFFSET(16) NUMBITS(8) [],
        /// Valid
        VLD OFFSET(0) NUMBITS(1) []
    ]
];

const BASE_ADDRESS: StaticRef<MpuRegisters> =
    unsafe { StaticRef::new(0x4000D000 as *const MpuRegisters) };

pub struct Mpu(StaticRef<MpuRegisters>);

impl Mpu {
    pub const unsafe fn new () -> Mpu {
        Mpu(BASE_ADDRESS)
    }
}

const APP_MEMORY_INDEX: usize = 1;

pub struct MK66Config {
    memory: Option<(u32, u32)>,
    regions: [Option<Region>; 11],
}

impl Default for MK66Config {
    fn default() -> MK66Config {
        MK66Config {
            memory: None,
            regions: [None; 11],
        }
    }
}

impl MK66Config {
    fn available_region_index(&self) -> Option<usize> {
        for (index, region) in self.regions.iter().enumerate() {
            if index == APP_MEMORY_INDEX {
                continue;
            }
            if let None = region {
                return Some(index);
            }
        }
        None
    }
}

struct Region {
    start: u32,
    end: u32,
    permissions: u32,
}

impl Region {
    fn new(
        start: u32,
        end: u32,
        permissions: Permissions,
    ) -> Region {
        let permissions = match permissions {
            Permissions::ReadWriteExecute => 0b111,
            Permissions::ReadWriteOnly => 0b110,
            Permissions::ReadExecuteOnly => 0b101,
            Permissions::ReadOnly => 0b100,
            Permissions::ExecuteOnly => 0b001,
        };

        Region {
            start: start,
            end: end,
            permissions: permissions,
        }
    }

    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn permissions(&self) -> u32 {
        self.permissions
    }
}

// Rounds `x` up to the nearest multiple of `y`.
fn round_up_to_nearest_multiple(x: u32, y: u32) -> u32 {
    if x % y == 0 {
        x
    } else {
        x + y - (x % y)
    }
}

impl mpu::MPU for Mpu {
    type MpuConfig = MK66Config;
    
    fn enable_mpu(&self) {
        let regs = &*self.0;
        regs.cesr.modify(ControlErrorStatus::VLD::Enable);
    }    
    
    fn disable_mpu(&self) {
        let regs = &*self.0;
        regs.cesr.modify(ControlErrorStatus::VLD::Disable);
    }

    fn number_total_regions(&self) -> usize {
        let regs = &*self.0;
        match regs.cesr.read(ControlErrorStatus::NRGD) {
            ControlErrorStatus::NRGD::Eight => 8,
            ControlErrorStatus::NRGD::Twelve => 12,
            ControlErrorStatus::NRGD::Sixteen => 16,
        }
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        // Regions must be aligned to 32 bytes
        let region_start = round_up_to_nearest_multiple(unallocated_memory_start as u32, 32);
        let region_size = round_up_to_nearest_multiple(min_region_size as u32, 32);

        let region_end = region_start + region_size;
        let unallocated_memory_end = (unallocated_memory_start as u32) + (unallocated_memory_size as u32);

        // Make sure we have enough memory for region 
        if region_end > unallocated_memory_end {
            return None;
        }
        
        let region = Region::new(region_start, region_end, permissions);
        
        let index = match config.available_region_index() {
            Some(index) => index,
            None => return None,
        };

        // Store region
        config.regions[index] = Some(region);

        Some((region_start as *const u8, region_size as usize))
    }

    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        let mut memory_size = {
            if min_memory_size < initial_app_memory_size + initial_kernel_memory_size {
                initial_app_memory_size + initial_kernel_memory_size
            } else {
                min_memory_size
            }
        };
        memory_size = round_up_to_nearest_multiple(memory_size as u32, 32);

        // Process memory block
        let memory_start = round_up_to_nearest_multiple(unallocated_memory_start as u32, 32);

        // MPU region for app-owned part
        let region_start = memory_start;
        let region_size = round_up_to_nearest_multiple(initial_app_memory_size as u32, 32);
        let region_end = region_start + region_size;

        // Make sure MPU region won't overlap kernel memory
        if region_size + (initial_kernel_memory_size as u32) > memory_size {
            memory_size += 32;
        }

        let memory_end = memory_start + memory_size;
        let unallocated_memory_end = (unallocated_memory_start as u32) + (unallocated_memory_size as u32);
        
        // Make sure we have enough memory for region 
        if memory_end > unallocated_memory_end {
            return None;
        }
        
        let region = Region::new(region_start, region_end, permissions);

        // Store region
        config.regions[APP_MEMORY_INDEX] = Some(region);

        Some((memory_start as *const u8, memory_size as usize))
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        // Check that region was actually created
        if let None = config.regions[APP_MEMORY_INDEX] {
            return Err(());
        }

        let (memory_start, memory_end) = match config.memory {
            Some((start, size)) => (start, size),
            None => return Err(()),
        };

        if memory_start % 32 != 0 || memory_end % 32 != 0 {
            return Err(());
        }

        // New region for app memory
        let region_start = memory_start;
        let region_end = round_up_to_nearest_multiple(app_memory_break as u32, 32);

        // Check if we have run out of memory 
        if region_end > (kernel_memory_break as u32) {
            return Err(());
        }
        
        let region = Region::new(region_start, region_end, permissions);

        // Store region
        config.regions[APP_MEMORY_INDEX] = Some(region);

        Ok(())
    }
    
    fn configure_mpu(&self, config: &Self::MpuConfig) {
        let regs = &*self.0;
        
        // On reset, region descriptor 0 is allocated to give full access to 
        // the entire 4 GB memory space to the core in both supervisor and user
        // mode, so we disable access for user mode
        regs.rgdaacs[0].0.modify(RegionDescriptorWord2::M0SM::ReadWriteExecute);
        regs.rgdaacs[0].0.modify(RegionDescriptorWord2::M0UM::CLEAR);

        // Write regions
        for (index, region) in config.regions.iter().enumerate() {
            // Region 0 is reserved
            let region_num = index + 1;

            match region {
                Some(region) => {
                    let start = region.start() >> 5;
                    let end = region.end() >> 5;
                    let user = region.permissions();

                    regs.rgds[region_num].rgd_word0.write(RegionDescriptorWord0::SRTADDR.val(start));
                    regs.rgds[region_num].rgd_word1.write(RegionDescriptorWord1::ENDADDR.val(end));
                    regs.rgds[region_num].rgd_word2.write(RegionDescriptorWord2::M0SM::SameAsUserMode + RegionDescriptorWord2::M0UM.val(user));
                    regs.rgds[region_num].rgd_word3.write(RegionDescriptorWord3::VLD::SET);
                },
                None => regs.rgds[region_num].rgd_word3.write(RegionDescriptorWord3::VLD::CLEAR),
            }
        }
    }
}
