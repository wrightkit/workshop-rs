use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Binding<T> {
    pub(crate) id: T,
    pub(crate) declared: bool,
}

impl ParseContext<'_> {
    pub(crate) fn condition_value(&mut self) -> Result<wir::ValueId> {
        self.value_with_condition(true)
    }

    pub(crate) fn value(&mut self) -> Result<wir::ValueId> {
        self.value_with_condition(false)
    }

    fn value_with_condition(&mut self, condition: bool) -> Result<wir::ValueId> {
        let saved = std::mem::replace(&mut self.in_condition, condition);
        let result = self.value_inner();
        self.in_condition = saved;
        result
    }

    pub(crate) fn with_call<T>(
        &mut self,
        call_id: &str,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let saved = std::mem::replace(&mut self.in_hud_text_call, call_id == "createHudText");
        let result = f(self);
        self.in_hud_text_call = saved;
        result
    }
}
