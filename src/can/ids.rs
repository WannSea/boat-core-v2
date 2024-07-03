use num_derive::FromPrimitive;


#[derive(FromPrimitive)]
pub enum CanIds {
    CanIdMostImportant=0x000,

    CanIdSolarPower = 0x026,
    CanIdSolarTemp = 0x022,

    CanIdMotor1 = 0x142,
    CanIdMotor2 = 0x143,

    //# to power off relays and batterys send 0x1 to power back on send 0x0
    CanIdPowerOff = 0x024,

    CanIdApmuTemp = 0x700,
    CanIdMpmuTemp = 0x702,

    CanIdPCSTemp = 0x724,
    CanIdLPMainPower = 0x726,



    // BMS RESERVED 0x8XX range

    CanIdFan1Rpm = 0x710,
    CanIdFan2Rpm = 0x712,
    CanIdFan3Rpm = 0x714,
    CanIdFan4Rpm = 0x716,

    CanIdMotorCurrent = 0x720,
    CanIdBattVoltage = 0x722,

    CanIdLeastImportant=0xFFE
}