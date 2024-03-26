use nmea0813::{Parser,ParseResult};

enum NMEAType {
    RMC,
    VTG,
    GGA,
    GLL
}

pub struct GPS {
    parser:Parser,
    gpsDataRaw:&[u8],
    pub gpsData:ParseResult,
    pub uart:UartConn
}

impl<U> GPS<U> {
    fn new<U> (uart:U) -> Self {
        parser:Parser::new(),
        uart:UartConn::init(GPS_PINS), //replace once UART protocol is finalized
        gpsData:newData()
    }
    fn newData() {
        while [gpsDataRaw.chars()[gpsDataRaw.chars().count() - 2], gpsDataRaw.chars()[gpsDataRaw.chars().count() - 1]] != "\r\n" {
            uart.read(gpsDataRaw)?;
        }
        gpsData = parser.parse_from_bytes(gpsDataRaw);
        gpsDataRaw = [0,gpsDataRaw.len()];
    }
    fn formatData(msgtype: &[NMEAType], interval:u8) {
        let mut cmd:[u8;16] = [160,161,0,9,8,msgtype.contains(NMEAType::GGA)*interval,0,0,msgtype.contains(NMEAType::GLL)*interval,msgtype.contains(NMEAType::RMC)*interval,msgtype.contains(NMEAType::VTG)*interval,0,8,13,10];
        uart.write(cmd)?;
    }
    
}