use strum_macros::Display;

pub enum BmsFunction {
    // The function of the BMS response in the format:
    // CAN ID: 0x<4 Bit: ID><16 Bit: Function>.
    // Example: 0x1024 contains the serial number of BMS with ID 1
    BmsIdV01_04 = 0x002,
    BmsIdV05_08 = 0x003,
    BmsIdV09_12 = 0x004,
    BmsIdV13_16 = 0x005,
    BmsIdV21_24 = 0x007,
    // BmsIdV25Bypass = 0x008,
    BmsIdT01_06 = 0x009,
    BmsIdInternalStatus1 = 0x020,
    BmsIdSerialNumberAnswer = 0x024,
    // BmsIdInfoIdConfig = 0x026,


    // PERIODICALLY SENT WITHOUT ID!
    EmsControl = 0x401,
    // GlobalStatus1 = 0x402,
    // GlobalStatus2 = 0x403,
    GlobalStatus3 = 0x404,
    GlobalStatus4 = 0x405,
    GlobalStatus5 = 0x406    
}

#[derive(Clone, Copy, Display)]
pub enum BmsIndividualRequestFunction {
    AllMeasurements = 3,
    InternalStatus1 = 101
}

pub enum EmsRequest {
    BmsGeneralRequest = 0x100,
    BmsIndividualRequest = 0x200
}

#[derive(Clone, Copy)]
pub struct BatteryPack {
    pub id: u8,
    pub serial_number: u32,
    pub part_number: u32
}
