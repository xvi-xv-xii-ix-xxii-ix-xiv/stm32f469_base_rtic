/// Trait для работы с GPIO пинами
pub trait GpioPin {
    /// Тип ошибки при работе с пином
    type Error;

    /// Устанавливает высокий уровень на пине
    fn set_high(&mut self) -> Result<(), Self::Error>;

    /// Устанавливает низкий уровень на пине
    fn set_low(&mut self) -> Result<(), Self::Error>;

    /// Проверяет, установлен ли высокий уровень на пине
    fn is_set_high(&self) -> Result<bool, Self::Error>;

    /// Проверяет, установлен ли низкий уровень на пине
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_set_high()?)
    }

    /// Переключает состояние пина
    fn toggle(&mut self) -> Result<(), Self::Error>;
}
