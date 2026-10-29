use esp_hal::Async;
use esp_hal::analog::adc::{Adc, AdcConfig, AdcPin, Attenuation};
use esp_hal::peripherals::{ADC1, GPIO2};
use log::debug;

const ADC_MAX: u32 = 4095;

#[derive(Clone, Copy)]
pub struct Reading {
    pub raw: u16,
    pub humidity: u8,
}

pub struct Sensor<'d> {
    adc: Adc<'d, ADC1<'d>, Async>,
    pin: AdcPin<GPIO2<'d>, ADC1<'d>>,
}

impl<'d> Sensor<'d> {
    pub fn new(adc: ADC1<'d>, pin: GPIO2<'d>) -> Self {
        debug!("Initializing ADC sensor on GPIO2 with 11 dB attenuation");

        let mut config = AdcConfig::new();
        let pin = config.enable_pin(pin, Attenuation::_11dB);
        let adc = Adc::new(adc, config).into_async();

        Self { adc, pin }
    }

    pub async fn read(&mut self) -> Reading {
        debug!("Starting ADC conversion");
        let raw = self.adc.read_oneshot(&mut self.pin).await;
        let humidity = ((raw as u32 * 100) / ADC_MAX) as u8;

        debug!("ADC conversion complete: raw={raw}, humidity={humidity}");

        Reading { raw, humidity }
    }
}
