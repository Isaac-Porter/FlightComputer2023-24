#![no_std]
#![no_main]

use core::mem::{self, MaybeUninit};

use bmp3::{hal::{Bmp3RawData, ReadBmp3, RegErrReg, RegStatus}, Bmp3Readout};
use defmt::*;
use embassy_executor::{task, Executor, Spawner};
use embassy_stm32::{bind_interrupts, dma::NoDma, gpio::{Level, Output, Speed}, peripherals::{self, DMA1_CH0, DMA1_CH1, PD8, PD9, USART3}, usart::{self, Config, Uart}};
use embassy_time::Timer;
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
    let usart3 = unsafe {
        USART3::steal()
    };

    let mut uart3 = unsafe {
        Uart::new(usart3, PD9::steal(), PD8::steal(), Irqs, DMA1_CH0::steal(), DMA1_CH1::steal(), Config::default())
    }.unwrap();

    loop {
        //info!("Sending 0x02");
        let buf = [0x02u8];
        uart3.blocking_write(&buf).unwrap();
        let mut response = [0u8; 256];
        let _ = uart3.read_until_idle(&mut response).await;
        info!("Read {:x}", response);
        Timer::after_millis(500).await;
    }

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