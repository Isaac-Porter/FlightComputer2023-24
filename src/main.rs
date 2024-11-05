#![no_std]
#![no_main]

use core::mem::{self, MaybeUninit};

use bmp3::{hal::{Bmp3RawData, ReadBmp3, RegErrReg, RegStatus}, Bmp3Readout};
use defmt::*;
use embassy_executor::{task, Executor, Spawner};
use embassy_stm32::{bind_interrupts, dma::NoDma, gpio::{Level, Output, Speed}, peripherals::{self, DMA1_CH0, DMA1_CH1, PD8, PD9, USART3}, usart::{self, Config, Uart}};
use embassy_time::Timer;
use rfm9x::ReadRfm9x;
use {defmt_rtt as _, panic_probe as _};
use sirin::Sirin;

unsafe fn transmute_into_static<T>(item: &mut T) -> &'static mut T {
    core::mem::transmute(item)
}

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    let mut executor = Executor::new();
    let executor: &'static mut Executor = transmute_into_static(&mut executor);
    let mut sirin = MaybeUninit::<Sirin>::uninit();
    let sirin = transmute_into_static(&mut sirin);
    executor.run(|spawner| {
        spawner.must_spawn(setup_task(spawner, sirin))
    })
}

#[task()]
async fn setup_task(spawner: Spawner, sirin: &'static mut MaybeUninit<Sirin>) {
    debug!("Begin Sirin init");

    let sirin = Sirin::init(sirin, spawner).await;

    debug!("End Sirin init");

    main_task(sirin).await
}

bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<peripherals::USART3>;
});

async fn main_task(sirin: &'static mut Sirin) {

    sirin.radio.set_mode(rfm9x::Mode::Sleep).await.unwrap();
    sirin.radio.set_mode(rfm9x::Mode::Stdby).await.unwrap();

    println!("Mode: {}", sirin.radio.mode().await.unwrap());

    println!("Version: 0x{:x}", sirin.radio.version().await.unwrap());

    loop {
        sirin.radio.transmit(&[0x80, 0x00, 0x01, 0x02, 0x03]).await.unwrap();

        println!("Sent msg");

        Timer::after_millis(500).await
    }

    /*let usart3 = unsafe {
        USART3::steal()
    };

    let mut uart3 = unsafe {
        Uart::new(usart3, PD9::steal(), PD8::steal(), Irqs, DMA1_CH0::steal(), DMA1_CH1::steal(), Config::default())
    }.unwrap();

    loop {
        let mut response = [0u8; 1024];
        let _ = uart3.read_until_idle(&mut response).await;

        let mut k = 0;
        while response[k] == 0xA0 && response[k + 1] == 0x0A1 {
            let len = ((response[k + 2] as u16) << 8) + response[k + 3] as u16;
            //info!("{:x}", &response[k..((len + 7) as usize + k)]);
            let id = response[k + 4];
            
            match id {
                0xDF => {
                    let msg = match response[k + 6] {
                        0x00 => "no fix",
                        0x01 => "fix prediction",
                        0x02 => "2d fix",
                        0x03 => "3d fix",
                        0x04 => "differential fix",
                        _ => "unknown"
                    };
                    
                    info!("fix status: {} ({})", msg, response[k + 6])
                },
                0xDE => {
                    let nsvs = response[k + 6];
                    let chsize = 8;
                    for i in 0..nsvs {
                        let offset = k + 7 + (chsize as usize * i as usize);
                        let chid = response[offset];
                        let svid = response[offset + 1];
                        let status = response[offset + 2];

                        let almanac = status & 0b001 > 0;
                        let ephemeris = status & 0b010 > 0;
                        let healthy = status & 0b100 > 0;

                        info!(
                            "GPS (chid: {}, svid: {}): [{}] almanac, [{}] ephemeris, [{}] healthy", 
                            chid, svid, almanac, ephemeris, healthy
                        )
                    }
                    info!("{} satellites total", nsvs)
                },
                _ => {}
            }

            k += len as usize + 7;
        }
    }*/

    /*loop {
        /*info!("Reading data...");
        let Bmp3RawData { raw_temperature, raw_pressure } = sirin.baro.io().read_raw_data().await.unwrap();
        info!("Read {} temperature and {} pressure", raw_temperature, raw_pressure);*/
        let Bmp3Readout { temperature, pressure } = sirin.baro.read().await.unwrap();
        info!("Read {} degrees Celsius at {} pascals", temperature.value, pressure.value);
        /*let err = sirin.baro.io().read_reg(RegErrReg).await.unwrap();
        info!("Err {:b}", err);
        let status = sirin.baro.io().read_reg(RegStatus).await.unwrap();
        info!("Status {:b}", status);*/
        Timer::after_millis(500).await;
    }*/
    
    /*loop {
        info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }*/
}