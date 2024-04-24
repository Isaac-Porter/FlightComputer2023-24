use crate::spi::spi::Spi;
use crate::spi::spi::SpiError;
use crate::spi::config::SpiConfig;
use embedded_hal_async::spi::SpiBus;

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

impl BaroStatus {
    fn new(val: u8) -> Self {
        Self {
            temp_rdy: (val & 0b01000000) == 64u8,
            pres_rdy: (val & 0b00100000) == 32u8,
            cmd_rdy: (val & 0b00010000) == 16u8
        }
    }
}

pub struct BaroDevError {
    conf_err: bool,
    cmd_err: bool,
    fatal_err: bool
}

impl BaroDevError {
    fn new(val: u8) -> Self {
        Self {
            conf_err:( val & 0b001) == 1u8,
            cmd_err:( val & 0b010) == 2u8,
            fatal_err:( val & 0b100) == 4u8
        }
    }
}

pub enum BaroError {
    MessageErr(SpiError),
    DeviceErr(BaroDevError)
}

pub struct Baro<S:SpiConfig> {
    spi: Spi<S>
}

impl <S: SpiConfig> Baro<S> {
    pub fn new(spi:Spi<S>) -> Self {
        Self {
            spi:spi
        }
    }
    async fn readRegister(&mut self, reg: BaroRegister) -> Result<u8,BaroError> { //Convert the buffer madness to something more functionalish
        let mut buf:[u8;1] = [0u8];
        let wbuf:[u8;1] = [reg as u8 | 0x80];
        if let Err(e) = self.spi.write(&wbuf).await { //Send over the address with read set to 1
            return Err(BaroError::MessageErr(e));
        }
        if let Err(e) = self.spi.read(&mut []).await {return Err(BaroError::MessageErr(e));} //Skip dummy byte
        if let Err(e) = self.spi.read(&mut buf).await { //Read actual data to buffer
            return Err(BaroError::MessageErr(e));
        }
        Ok(buf[0])
    }
    async fn read3(&mut self, reg: BaroRegister) -> Result<[u8;4],BaroError> { //The BMP388 can read consecutive addresses without sending each address, so I added a block implementation
        let buf:[u8;4] = [0u8;4];
        let wbuf:[u8;1] = [reg as u8 | 0x80];
        if let Err(e) = self.spi.write(&wbuf).await {
            return Err(BaroError::MessageErr(e));
        }
        if let Err(e) = self.spi.read(&mut []).await {return Err(BaroError::MessageErr(e));}
        for i in 0..2 {
            if let Err(e) = self.spi.read(&mut [buf[i]]).await {
                return Err(BaroError::MessageErr(e));
            }
        }
        Ok(buf)
    }
    pub async fn temp(&mut self) -> Result<u32,BaroError> { //Get the temperature in whatever units the BMP is using idc
        match self.readRegister(BaroRegister::ErrReg).await {
            Ok(n) => {
                match n {
                    0u8 => Ok(u32::from_le_bytes(self.read3(BaroRegister::Data3).await?)),
                    _ => Err(BaroError::DeviceErr(BaroDevError::new(n)))
                }
            }
            Err(err) => Err(err)
        }
    }
    pub async fn pres(&mut self) -> Result<u32,BaroError> {
        match self.readRegister(BaroRegister::ErrReg).await {
            Ok(n) => {
                match n {
                    0u8 => {
                        Ok(u32::from_le_bytes(self.read3(BaroRegister::Data0).await?))
                    }
                    _ => Err(BaroError::DeviceErr(BaroDevError::new(n)))
                }
            }
            Err(err) => Err(err)
        }
    }
    pub async fn sensorTime(&mut self) -> Result<u32,BaroError> {
        match self.readRegister(BaroRegister::ErrReg).await {
            Ok(n) => {
                match n {
                    0u8 => {
                        Ok(u32::from_le_bytes(self.read3(BaroRegister::SensorTime0).await?))
                    }
                    _ => Err(BaroError::DeviceErr(BaroDevError::new(n)))
                }
            }
            Err(err) => Err(err)
        }
    }
}