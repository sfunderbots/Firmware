use embedded_hal::blocking::spi::Write;
use ftdi_embedded_hal::ftdi_mpsse::{ClockData, ClockDataOut, MpsseCmdBuilder, MpsseCmdExecutor};
use ftdi_embedded_hal::{Spi, SpiDevice, Error};


const MAX_TRANSFER: usize = 1500;

pub struct BufferedSpi<Device: MpsseCmdExecutor> 
{
    spi: Spi<Device>,
    buffer: Vec<u8>,
}

impl<Device, E> BufferedSpi<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub fn new(spi: Spi<Device>) -> Self {
        BufferedSpi { spi, buffer: Vec::with_capacity(MAX_TRANSFER) }
    }
}

impl<Device, E> Write<u8> for BufferedSpi<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;
    fn write(&mut self, words: &[u8]) -> Result<(), Error<E>> {
        self.buffer.extend_from_slice(words);

        if self.buffer.len() >= MAX_TRANSFER {
            println!("Buffer full, writing {} bytes", self.buffer.len());
            self.spi.write(&self.buffer[0..MAX_TRANSFER])?;
            self.buffer.clear();
        }
        Ok(())
    }
}
