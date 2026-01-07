use esp_idf_hal::gpio::{Input, InputPin, OutputPin, PinDriver};

pub struct Button<P>
where
    P: InputPin + OutputPin,
{
    pin: PinDriver<'static, P, Input>,
    active_low: bool,
}
impl<P> Button<P>
where
    P: InputPin + OutputPin,
{
    pub fn new(pin: PinDriver<'static, P, Input>) -> Self {
        Button { pin, active_low: true }
    }

    pub fn is_pressed(&mut self) -> bool {
        let level = self.pin.is_high();
        if self.active_low {
            !level
        } else {
            level
        }
    }
}
