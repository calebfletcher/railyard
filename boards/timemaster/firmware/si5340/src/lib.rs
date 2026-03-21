#![no_std]

use embedded_hal::i2c::Operation;

pub struct Si5340<I: RegisterInterface> {
    interface: I,
}

impl<I2C: embedded_hal::i2c::I2c> Si5340<I2c<I2C>> {
    pub fn new_i2c(i2c: I2C, addr: Address) -> Self {
        let mut device = Self {
            interface: I2c { i2c, addr },
        };

        assert!(device.device_ready());
        assert_eq!(device.part_number(), 5340);

        device
    }
}

impl<I: RegisterInterface> Si5340<I> {
    pub fn register(&mut self, register: u16) -> u8 {
        self.interface.read_single(register as u8)
    }

    /// Read the part number from the device, should be 5340 to the Si5340.
    pub fn part_number(&mut self) -> u16 {
        let mut bytes = [0; 2];
        self.interface.read_multi(0x0002, &mut bytes);

        // convert to u16
        let chars = [bytes[1] >> 4, bytes[1] & 0xF, bytes[0] >> 4, bytes[0] & 0xF];
        chars[0] as u16 * 1000 + chars[1] as u16 * 100 + chars[2] as u16 * 10 + chars[3] as u16
    }

    /// Check whether the device is ready for read/write access to its registers.
    pub fn device_ready(&mut self) -> bool {
        // This register exists on all pages, no need to switch pages
        self.interface.read_single(0xFE) == 0x0F
    }
}

pub struct Address(pub u8);

impl Address {
    /// Calculate the I2C address of the device based on the known state of
    /// the a0 and a1 pins.
    pub fn from_pins(a0: bool, a1: bool) -> Self {
        Self(0b1110100 | (a1 as u8) << 1 | a0 as u8)
    }
}

pub trait RegisterInterface {
    /// Write a single byte to a register
    fn write_single(&mut self, register: u8, data: u8);
    /// Write a series of bytes from the given start register, auto-incrementing
    /// the register
    fn write_multi(&mut self, start_register: u8, data: &[u8]);

    /// Read a single byte from a register
    fn read_single(&mut self, register: u8) -> u8;
    /// Read a series of bytes from the given start register, auto-incrementing
    /// the register
    fn read_multi(&mut self, start_register: u8, data: &mut [u8]);
}

pub struct I2c<I: embedded_hal::i2c::I2c> {
    i2c: I,
    addr: Address,
}

impl<I: embedded_hal::i2c::I2c> RegisterInterface for I2c<I> {
    fn write_single(&mut self, register: u8, data: u8) {
        self.i2c.write(self.addr.0, &[register, data]).unwrap();
    }

    fn write_multi(&mut self, start_register: u8, data: &[u8]) {
        self.i2c
            .transaction(
                self.addr.0,
                &mut [Operation::Write(&[start_register]), Operation::Write(data)],
            )
            .unwrap();
    }

    fn read_single(&mut self, register: u8) -> u8 {
        // Per family reference manual, reads are two separate stages (no repeated start)
        let mut value = [0];
        self.i2c.write(self.addr.0, &[register]).unwrap();
        self.i2c.read(self.addr.0, &mut value).unwrap();
        value[0]
    }

    fn read_multi(&mut self, start_register: u8, data: &mut [u8]) {
        // Per family reference manual, reads are two separate stages (no repeated start)
        self.i2c.write(self.addr.0, &[start_register]).unwrap();
        self.i2c.read(self.addr.0, data).unwrap();
    }
}
