#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m::asm;
use cortex_m_rt::{entry, exception};
// use cortex_m_semihosting::hprintln;
use stm32f1::stm32f103;
use stm32f1::stm32f103::Peripherals;

static mut SYSTICK_QUEUED: bool = false;

// ----------------------------------------------------------------------------

#[entry]
fn main() -> ! {
    // hprintln!("-------->> HELLO RUST! <<--------").unwrap();
    let peripherals = stm32f103::Peripherals::take().unwrap();
    config_gpio(&peripherals);
    config_stk(&peripherals);
    config_uart(&peripherals);

    let gpioa = &peripherals.GPIOA;
    let mut led_state = false;

    loop {
        unsafe {
            while !SYSTICK_QUEUED { asm::nop(); } // do not optimize this loop in release
            SYSTICK_QUEUED = false
        }
        led_state = !led_state;
        gpioa.odr.write(|w| match led_state {
            true => w.odr5().set_bit(),
            false => w.odr5().clear_bit(),
        }); 
    }
}

// ----------------------------------------------------------------------------

fn config_gpio(p: &Peripherals) {
    p.RCC.apb2enr.modify(|_, w| w.iopaen().set_bit()); // enable clock for GPIOA
    p.GPIOA.crl.modify(|_, w| w
        // LED
        .mode5().bits(0b01) // 10MHz
        .cnf5().bits(0b00) // push-pull ouptut
    );
}

fn config_stk(p: &Peripherals) {
    p.STK.ctrl.modify(|_, w| w
        .enable().set_bit()
        .tickint().set_bit()
        .clksource().set_bit()
        .countflag().set_bit()
    );
    p.STK.load_.modify(|_, w| unsafe {
        w.reload().bits(1_000_000)
    });
}

fn config_uart(p: &Peripherals) {
    p.RCC.apb1enr.modify(|_, w| w.usart2en().set_bit()); // enable clock for USART2
    p.GPIOA.crl.modify(|_, w| w
        // USART2 TX
        .mode2().bits(0b01) // 10MHz
        .cnf2().bits(0b10) // alternae push-pull

        // USART2 RX
        .mode3().bits(0b00) // Reserved
        .cnf3().bits(0b10) // Input floating
    );
    p.GPIOA.odr.modify(|_, w| w
        // USART RX
        .odr3().set_bit() // Input pull-up
    );
    p.USART2.brr.modify(|_, w| w
        .div_fraction().bits(0)
        .div_mantissa().bits(0)
    );
    p.USART2.cr1.modify(|_, w| w
        .ue().set_bit()
    );
    // p.USART2.cr2.modify(f)
}

#[exception]
fn SysTick() {
    unsafe { SYSTICK_QUEUED = true };
}