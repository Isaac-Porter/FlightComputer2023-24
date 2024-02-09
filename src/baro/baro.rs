use spi_device::SpiDevice;
use lib::Peripherals;

enum BaroRegister {
    ChipID = 0x00,
    ErrReg = 0x02,
    Status = 0x03,
    Data0 = 0x04,
    Data1 = 0x05,
    Data2 = 0x06,
    Data3 = 0x07,
    Data4 = 0x08,
    Data5 = 0x09,
    SensorTime0 = 0x0C,
    SensorTime1 = 0x0D,
    SensorTime2 = 0x0E,
    Event = 0x10,
    IntStatus = 0x11,
    FifoLength0 = 0x12,
    FifoLength1 = 0x13,
    FifoData = 0x14,
    FifoWtm0 = 0x15,
    FifoWtm1 = 0x16,
    FifoConfig1 = 0x17,
    FifoConfig2 = 0x18,
    IntCtrl = 0x19,
    IfConf = 0x1A,
    PwrCtrl = 0x1B,
    OSR = 0x1C,
    ODR = 0x1D,
    Config = 0x1F,
    Cmd = 0x7E
}

struct BaroStatus {
    temp_rdy: bool,
    pres_rdy: bool,
    cmd_rdy: bool
}

impl From<u8> for BaroStatus {
    fn from(in:u8) -> BaroStatus {
        BaroStatus {
            temp_rdy: (in & b01000000) == 64u8,
            pres_rdy: (in & b00100000) == 32u8,
            cmd_rdy: (in & b00010000) == 16u8
        }
    }
}

pub struct BaroDevError {
    conf_err: bool,
    cmd_err: bool,
    fatal_err: bool
}

impl From<u8> for BaroDevError {
    fn from(in: u8) -> BaroDevError {
        BaroDevError {
            conf_err:(in & b001) == 1u8,
            cmd_err:(in & b010) == 2u8,
            fatal_err:(in & b100) == 4u8
        }
    }
}

pub enum BaroError {
    MessageErr(SpiError),
    DeviceErr(BaroDevError)
}

pub struct Baro {
    spi: SpiInterface;
}

impl Baro {
    fn new(conf:SpiConfig) -> Self {
        Self {
            spi:SpiInterface::generate(BARO_PIN); //Replace this with actual SPI device registration
        }
    }
    fn readRegister(reg: BaroRegister) -> Result<u8,BaroError> { //Convert the buffer madness to something more functionalish
        let mut buf:u8 = 0u8;
        if let Err(e) = spi.write(reg | 0x80) { //Send over the address with read set to 1
            Err(BaroError::MessageErr(e))
        }
        if let Err(e) = spi.read(()) {Err(BaroError::MessageErr(e))} //Skip dummy byte
        if let Err(e) = spi.read(buf) { //Read actual data to buffer
            Err(BaroError::MessageErr(e))
        }
        Ok(buf)
    }
    fn readConsecutive(reg: BaroRegister, num: u32) -> Result<&[u8],BaroError> { //The BMP388 can read consecutive addresses without sending each address, so I added a block implementation
        let mut buf:[u8,num] = [0u8,num];
        if let Err(e) = spi.write(reg | 0x80) {
            Err(BaroError::MessageErr(e))
        }
        if let Err(e) = spi.read(()) {Err(BaroError::MessageErr(e))}
        for i in 0..(num - 1) {
            if let Err(e) = spi.read(buf[i]) {
                Err(BaroError::MessageErr(e))
            }
        }
        Ok(buf)
    }
    pub fn temp() -> Result<u32,BaroError> { //Get the temperature in whatever units the BMP is using idc
        match readRegister(BaroRegister::ErrReg) {
            Ok(n) => {
                match n {
                    0u8 => {
                        Ok(u32::from_le_bytes(readConsecutive(BaroRegister::Data3,3)))
                    }
                    _ => Err(BaroError::DeviceErr(BaroDevError::from(n)))
                }
            }
            Err(err) => Err(BaroError::MessageErr(err))
        }
    }
    pub fn pres() -> Result<u32,BaroError> {
        match readRegister(BaroRegister::ErrReg) {
            Ok(n) => {
                match n {
                    0u8 => {
                        Ok(u32::from_le_bytes(readConsecutive(BaroRegister::Data0,3)))
                    }
                    _ => Err(BaroError::DeviceErr(BaroDevError::from(n)))
                }
            }
            Err(err) => Err(BaroError::MessageErr(err))
        }
    }
    pub fn sensorTime() -> Result<u32,BaroError> {
        match readRegister(BaroRegister::ErrReg) {
            Ok(n) => {
                match n {
                    0i8 => {
                        Ok(u32::from_le_bytes(readConsecutive(BaroRegister::SensorTime0,3)))
                    }
                    _ => Err(BaroError::DeviceErr(BaroDevError::from(n)))
                }
            }
            Err(err) => Err(BaroError::MessageErr(err))
        }
    }
}