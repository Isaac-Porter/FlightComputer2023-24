use core::future;
//use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiError {
    SelectError,
    WriteError,
    ReadError,
}

pub trait SpiDeviceConn where
    Self: Sized,
{
    type Future<Out>: future::Future<Output = Out> + Send;
    type Selected: SelectedSpiDeviceConn<SpiDeviceConn = Self>;
    
    /// Automatically selects the device and then swaps the bytes.
    /// TODO: figure out how to implement this. Async is not allowed in traits
    /*async fn write(&mut self, data: impl Iterator<Item = u8> + Send)
        -> Result<(), SpiError> {
        let selected = self.select().await.map_err(|_| SpiError::SelectError)?;

        for byte in data {
            selected.write_byte(byte).await.map_err(|_| SpiError::WriteError)?;
        }

        *self = selected.deselect().await.map_err(|_| SpiError::SelectError)?;
        Ok(())
    }*/
    fn write(&mut self, data: impl Iterator<Item = u8> + Send)
        -> Self::Future<Result<(), SpiError>>;

    /// Automatically selects this device, then reads `num_bytes` bytes from the device.
    fn read<const num_bytes: usize>(&mut self)
        -> Self::Future<Result<[u8; num_bytes], SpiError>>;

    /// Automatically selects this device, then swaps the bytes in the `data` buffer. 
    /// So each byte in the buffer is
    /// sent to the device, and the buffer is modified in-place with the returned bytes.
    fn swap<const num_bytes: usize>(&mut self, data: &[u8; num_bytes])
        -> Self::Future<Result<(), SpiError>>;

    fn select(self) -> Self::Future<Result<Self::Selected, SpiError>>;
}

pub trait SelectedSpiDeviceConn: Send {
    type SpiDeviceConn: SpiDeviceConn;
    fn deselect(self)
        -> <Self::SpiDeviceConn as SpiDeviceConn>
        ::Future<
            Result<
                Self::SpiDeviceConn,
                SpiError
            >
        >;

    fn write_byte(&mut self, byte: u8)
        -> <Self::SpiDeviceConn as SpiDeviceConn>
        ::Future<
            Result<
                (),
                SpiError
            >
        >;

    fn read_byte(&mut self)
        -> <Self::SpiDeviceConn as SpiDeviceConn>
        ::Future<
            Result<
                u8,
                SpiError
            >
        >;
}