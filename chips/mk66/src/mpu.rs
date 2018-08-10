//! Implementation of the MK66 memory protection unit.
//!
//! - Author: Conor McAvity <cmcavity@stanford.edu>

use kernel::common::regs::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::mpu;

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
            ReadWriteExecute = 0b00,
            ReadExecuteOnly = 0b01,
            ReadWriteOnly = 0b10,
            SameAsUserMode = 0b11 
        ],
        /// Bus Master 3 User Mode Access Control
        M3UM OFFSET(18) NUMBITS(3) [],
        /// Bus Master 2 Process Identifier Enable
        M2PE OFFSET(17) NUMBITS(1) [],
        /// Bus Master 2 Supervisor Mode Access Control
        M2SM OFFSET(15) NUMBITS(2) [
            ReadWriteExecute = 0b00,
            ReadExecuteOnly = 0b01,
            ReadWriteOnly = 0b10,
            SameAsUserMode = 0b11 
        ],
        /// Bus Master 2 User Mode Access Control 
        M2UM OFFSET(12) NUMBITS(3) [],
        /// Bus Master 1 Process Identifier Enable
        M1PE OFFSET(11) NUMBITS(1) [],
        /// Bus Master 1 Supervisor Mode Access Control
        M1SM OFFSET(9) NUMBITS(2) [
            ReadWriteExecute = 0b00,
            ReadExecuteOnly = 0b01,
            ReadWriteOnly = 0b10,
            SameAsUserMode = 0b11 
        ],
        /// Bus Master 1 User Mode Access Control
        M1UM OFFSET(6) NUMBITS(3) [],
        /// Bus Master 0 Process Identifier Enable
        M0PE OFFSET(5) NUMBITS(1) [],
        /// Bus Master 0 Supervisor Mode Access Control
        M0SM OFFSET(3) NUMBITS(2) [
            ReadWriteExecute = 0b00,
            ReadExecuteOnly = 0b01,
            ReadWriteOnly = 0b10,
            SameAsUserMode = 0b11 
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

impl mpu::MPU for Mpu {
    fn enable_mpu(&self) {
        let regs = &*self.0;

        // On reset, region descriptor 0 is allocated to give full access to 
        // the entire 4 GB memory space to the core in both supervisor and user
        // mode, so we disable access for user mode
        //regs.rgdaacs[0].0.modify(RegionDescriptorWord2::M0SM::ReadWriteExecute);
        //regs.rgdaacs[0].0.modify(RegionDescriptorWord2::M0UM::CLEAR);

        regs.cesr.modify(ControlErrorStatus::VLD::Enable);
    }    
    
    fn disable_mpu(&self) {
        let regs = &*self.0;
        regs.cesr.modify(ControlErrorStatus::VLD::Disable);
    }

    fn create_region(
        region_num: usize,
        start: usize,
        len: usize,
        execute: mpu::ExecutePermission,
        access: mpu::AccessPermission,
    ) -> Option<mpu::Region> {
        // First region is reserved
        let region_num = region_num + 1;

        if region_num > 11 || start % 32 != 0 || len % 32 != 0 {
            return None;
        }

        // Ignore supervisor permissions
        let user = match (access, execute) {
            (mpu::AccessPermission::NoAccess, mpu::ExecutePermission::ExecutionPermitted) => 0b001,
            (mpu::AccessPermission::NoAccess, mpu::ExecutePermission::ExecutionNotPermitted) => 0b000,
            (mpu::AccessPermission::PrivilegedOnly, mpu::ExecutePermission::ExecutionPermitted) => 0b001, 
            (mpu::AccessPermission::PrivilegedOnly, mpu::ExecutePermission::ExecutionNotPermitted) => 0b000,
            (mpu::AccessPermission::UnprivilegedReadOnly, mpu::ExecutePermission::ExecutionPermitted) => 0b101, 
            (mpu::AccessPermission::UnprivilegedReadOnly, mpu::ExecutePermission::ExecutionNotPermitted) => 0b100,
            (mpu::AccessPermission::ReadWrite, mpu::ExecutePermission::ExecutionPermitted) => 0b111,
            (mpu::AccessPermission::ReadWrite, mpu::ExecutePermission::ExecutionNotPermitted) => 0b110, 
            (mpu::AccessPermission::Reserved, mpu::ExecutePermission::ExecutionPermitted) => return None,
            (mpu::AccessPermission::Reserved, mpu::ExecutePermission::ExecutionNotPermitted) => return None, 
            (mpu::AccessPermission::PrivilegedOnlyReadOnly, mpu::ExecutePermission::ExecutionPermitted) => 0b001, 
            (mpu::AccessPermission::PrivilegedOnlyReadOnly, mpu::ExecutePermission::ExecutionNotPermitted) => 0b000, 
            (mpu::AccessPermission::ReadOnly, mpu::ExecutePermission::ExecutionPermitted) => 0b101, 
            (mpu::AccessPermission::ReadOnly, mpu::ExecutePermission::ExecutionNotPermitted) => 0b100, 
            (mpu::AccessPermission::ReadOnlyAlias, mpu::ExecutePermission::ExecutionPermitted) => 0b101, 
            (mpu::AccessPermission::ReadOnlyAlias, mpu::ExecutePermission::ExecutionNotPermitted) => 0b100, 
        };

        // With the current interface, we have to pack all the region configuration into these 2 words
        let base_address = (start | region_num) as u32;   
        let attributes = ((start + len) | user) as u32;

        let region = unsafe { mpu::Region::new(base_address, attributes) };

        Some(region)
    }

    fn set_mpu(&self, region: mpu::Region) {
        let regs = &*self.0;

        let base_address = region.base_address();
        let attributes = region.attributes();

        let start = base_address >> 5; 
        let region_num = (base_address & 0x1f) as usize;
        let end = attributes >> 5;
        let user = attributes & 0x7;

        let num = regs.rgds[0].rgd_word1.read(RegionDescriptorWord1::ENDADDR);
        debug!("Num: {:#X}", num);

        // Write to region descriptor
        //regs.rgds[region_num].rgd_word0.write(RegionDescriptorWord0::SRTADDR.val(start));
        //regs.rgds[region_num].rgd_word1.write(RegionDescriptorWord1::ENDADDR.val(end));
        //regs.rgds[region_num].rgd_word2.write(RegionDescriptorWord2::M0SM::SameAsUserMode + RegionDescriptorWord2::M0UM.val(user));
        //regs.rgds[region_num].rgd_word3.write(RegionDescriptorWord3::VLD::SET);
    }
}
