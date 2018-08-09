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
    ers: [MpuErrorRegisters; 5],
    rgds: [MpuRegionDescriptor; 12],
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
        /// Bus Master 3 User Mode Access Control - Execute 
        M3UM_X OFFSET(20) NUMBITS(1) [],
        /// Bus Master 3 User Mode Access Control - Write 
        M3UM_W OFFSET(19) NUMBITS(1) [],
        /// Bus Master 3 User Mode Access Control - Read 
        M3UM_R OFFSET(18) NUMBITS(1) [],
        /// Bus Master 2 Process Identifier Enable
        M2PE OFFSET(17) NUMBITS(1) [],
        /// Bus Master 2 Supervisor Mode Access Control
        M2SM OFFSET(15) NUMBITS(2) [
            ReadWriteExecute = 0,
            ReadExecuteOnly = 1,
            ReadWriteOnly = 2,
            SameAsUserMode = 3
        ],
        /// Bus Master 2 User Mode Access Control - Execute 
        M2UM_X OFFSET(14) NUMBITS(1) [],
        /// Bus Master 2 User Mode Access Control - Write 
        M2UM_W OFFSET(13) NUMBITS(1) [],
        /// Bus Master 2 User Mode Access Control - Read 
        M2UM_R OFFSET(12) NUMBITS(1) [],
        /// Bus Master 1 Process Identifier Enable
        M1PE OFFSET(11) NUMBITS(1) [],
        /// Bus Master 1 Supervisor Mode Access Control
        M1SM OFFSET(9) NUMBITS(2) [
            ReadWriteExecute = 0,
            ReadExecuteOnly = 1,
            ReadWriteOnly = 2,
            SameAsUserMode = 3
        ],
        /// Bus Master 1 User Mode Access Control - Execute 
        M1UM_X OFFSET(8) NUMBITS(1) [],
        /// Bus Master 1 User Mode Access Control - Write 
        M1UM_W OFFSET(7) NUMBITS(1) [],
        /// Bus Master 1 User Mode Access Control - Read 
        M1UM_R OFFSET(6) NUMBITS(1) [],
        /// Bus Master 0 Process Identifier Enable
        M0PE OFFSET(5) NUMBITS(1) [],
        /// Bus Master 0 Supervisor Mode Access Control
        M0SM OFFSET(3) NUMBITS(2) [
            ReadWriteExecute = 0,
            ReadExecuteOnly = 1,
            ReadWriteOnly = 2,
            SameAsUserMode = 3
        ],
        /// Bus Master 0 User Mode Access Control - Execute 
        M0UM_X OFFSET(2) NUMBITS(1) [],
        /// Bus Master 0 User Mode Access Control - Write 
        M0UM_W OFFSET(1) NUMBITS(1) [],
        /// Bus Master 0 User Mode Access Control - Read 
        M0UM_R OFFSET(0) NUMBITS(1) []
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
        regs.rgdaacs[0].0.modify(RegionDescriptorWord2::M0SM::ReadWriteExecute);
        regs.rgdaacs[0].0.modify(RegionDescriptorWord2::M0UM_R::CLEAR 
                                 + RegionDescriptorWord2::M0UM_W::CLEAR 
                                 + RegionDescriptorWord2::M0UM_X::CLEAR);

        regs.cesr.modify(ControlErrorStatus::VLD::Enable);
    }    
    
    fn disable_mpu(&self) {
        let regs = &*self.0;
        regs.cesr.modify(ControlErrorStatus::VLD::Disable);
    }

    fn create_region(
        _: usize,
        _: usize,
        _: usize,
        _: mpu::ExecutePermission,
        _: mpu::AccessPermission,
    ) -> Option<mpu::Region> {
        Some(mpu::Region::empty(0))  
    }

    fn set_mpu(&self, _: mpu::Region) {}
}
